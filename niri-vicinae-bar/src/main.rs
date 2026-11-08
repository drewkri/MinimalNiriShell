use json::JsonValue;
use log::error;
use std::{collections::HashMap, ffi::OsStr, fs, process::Command};

use chrono::Local;
use gtk4::{
    Button, CssProvider, IconTheme, Image, Label, PopoverMenu,
    gdk::Display,
    gio::{
        self, ActionEntry, DataInputStream, Subprocess, SubprocessFlags,
        prelude::{
            ActionMapExt, ActionMapExtManual, ApplicationExt, ApplicationExtManual,
            DataInputStreamExtManual,
        },
        resources_register_include,
    },
    glib::{self, OptionFlags},
    prelude::{BoxExt, ButtonExt, GtkApplicationExt, GtkWindowExt, PopoverExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use libadwaita::{Application, prelude::AdwApplicationWindowExt};

use crate::settings::{Config, open_settings_window};

const APP_ID: &str = "io.github.drewkri.niri-vicinae-bar";

mod settings;

fn main() -> glib::ExitCode {
    env_logger::init();
    let _ = resources_register_include!("niri-vicinae-hud.gresource");
    let app = Application::builder().application_id(APP_ID).build();
    app.add_main_option(
        "settings",
        b's'.into(),
        OptionFlags::NO_ARG,
        glib::OptionArg::None,
        "Open the settings menu instead of starting the bar",
        None,
    );
    // startup tasks
    app.connect_startup(|_| {
        let display = Display::default().expect("No display connected.");

        // Handle loading CSS
        let mut path = Config::config_path();
        path.push("style.css");
        if let Ok(css) = fs::read_to_string(path) {
            let provider = CssProvider::new();
            provider.load_from_data(&css);
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        // load icons
        let theme = IconTheme::for_display(&display);
        theme.add_resource_path("/hud/icons");
    });
    app.connect_handle_local_options(|app, opts| {
        let settings = opts
            .lookup::<bool>("settings")
            .ok()
            .and_then(|inner| inner)
            .is_some_and(|b| b);
        if settings {
            app.connect_activate(open_settings_window);
        } else {
            app.connect_activate(activate);
        }
        std::ops::ControlFlow::Continue(())
    });

    app.run();

    glib::ExitCode::SUCCESS
}

fn activate(app: &Application) {
    let cfg = Config::try_load();
    let bar = build_bar(app, &cfg);
    bar.set_size_request(1, 1);
    bar.show();

    app.add_action_entries([ActionEntry::builder("reload-config")
        .activate(glib::clone!(
            #[weak]
            app,
            move |_, _, _| {
                let cfg = Config::try_load();
                for win in app.windows() {
                    // close the two layer shell panels and reopen them.
                    if win.is_layer_window() {
                        win.close();
                    }
                }
                let bar = build_bar(&app, &cfg);
                bar.set_size_request(1, 1);
                // Bottom first, as it has the corner contents
                bar.show();
            }
        ))
        .build()]);
}

fn build_bar(app: &Application, cfg: &Config) -> libadwaita::ApplicationWindow {
    let window = libadwaita::ApplicationWindow::new(app);

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Top);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
    if cfg.bar_on_top {
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    } else {
        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    }
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.auto_exclusive_zone_enable();

    let main_box = gtk4::CenterBox::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(true)
        .margin_bottom(4)
        .margin_top(4)
        .margin_end(6)
        .margin_start(6)
        .build();
    window.set_content(Some(&main_box));

    let buttons_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(false)
        .halign(gtk4::Align::Center)
        .spacing(24)
        // .margin_bottom(4)
        // .margin_end(4)
        // .margin_start(4)
        // .margin_top(4)
        .build();
    let overview_button = gtk4::Button::builder()
        .icon_name("tabs-stack-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    let launcher_button = gtk4::Button::builder()
        .icon_name("open-menu-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    overview_button.connect_clicked(|_| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("open-overview")
            .spawn();
    });
    launcher_button.connect_clicked(|_| {
        let _ = Command::new("vicinae").arg("open").spawn();
    });
    buttons_box.append(&overview_button);
    buttons_box.append(&launcher_button);

    if cfg.show_keyboard_button {
        let keyboard_button = gtk4::Button::builder()
            .icon_name("input-keyboard-symbolic")
            .valign(gtk4::Align::Center)
            .hexpand(false)
            .css_classes(["circular"])
            .build();
        // moved into closure
        let keyboard_command = cfg.keyboard_button_command.clone();
        keyboard_button.connect_clicked(move |_| {
            parse_and_run_command(&keyboard_command);
        });
        buttons_box.append(&keyboard_button);
    }

    // Central current window button
    let app_lbl = gtk4::Label::builder()
        .label("")
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .hexpand(true)
        .visible(false)
        .build();
    let app_icon = gtk4::Image::builder().icon_name("").visible(false).build();
    let center_btn_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .hexpand(true)
        .build();
    center_btn_box.append(&app_icon);
    center_btn_box.append(&app_lbl);
    let center_btn = gtk4::Button::builder()
        .hexpand(false)
        .css_classes(["flat"])
        .child(&center_btn_box)
        .build();
    // local thread which will handle talking to niri and getting the currently focused window
    glib::spawn_future_local(glib::clone!(
        #[weak]
        app_lbl,
        #[weak]
        app_icon,
        async move { update_window_display(app_lbl, app_icon).await }
    ));
    add_app_menu_popover(app, &center_btn);

    // Date/time/battery indicator
    let time_lbl = gtk4::Label::builder()
        .label("")
        .css_classes(["numeric", "title-4"])
        // .css_classes(["numeric"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .hexpand(true)
        .build();

    let fmt_str_clone = cfg.time_format.clone();
    // will move into closure
    let show_battery = cfg.battery;
    glib::spawn_future_local(glib::clone!(
        #[weak]
        time_lbl,
        async move {
            loop {
                let now = Local::now();
                // refresh everything
                let mut text = String::new();
                if show_battery && let Some((bat_str, bat_num)) = try_get_battery() {
                    let icon = match bat_num {
                        0..15 => "",
                        15..40 => "",
                        40..60 => "",
                        60..85 => "",
                        85..=100 => "",
                        _ => "", // should be unreachable, i hope, unless your computer is crazy
                    };
                    // battery icon needs extra padding
                    text.push_str(&format!("{}   {}% | ", icon, bat_str));
                }
                text.push_str(&now.format(&fmt_str_clone).to_string());

                time_lbl.set_text(&text);
                time_lbl.set_tooltip_text(Some(&now.format("%A, %B%e").to_string()));
                glib::timeout_future_seconds(1).await;
            }
        }
    ));
    let status_btn = gtk4::Button::builder()
        .child(&time_lbl)
        .hexpand(false)
        .css_classes(["flat"])
        .build();
    status_btn.connect_clicked(|_| {
        let _ = Command::new("vicinae")
            .arg("vicinae://launch/@drewkri/quick-menu/quick-menu")
            .spawn();
    });

    main_box.set_start_widget(Some(&buttons_box));
    main_box.set_center_widget(Some(&center_btn));
    main_box.set_end_widget(Some(&status_btn));

    window
}

fn parse_and_run_command(cmd: &str) {
    let mut iter = cmd.split(' ');
    if let Some(n) = iter.next() {
        let mut cmd = Command::new(n);
        for arg in iter {
            cmd.arg(arg);
        }
        let _ = cmd.spawn();
    }
}

/// Gets the battery. Blocking on the cat command which really shouldn't block for very long
/// but it may still block.
fn try_get_battery() -> Option<(String, u32)> {
    // Tries both BAT0 and BAT1 since the main battery could be either.
    for i in 0..=1 {
        if let Some(battery) = try_get_battery_index(i) {
            let trimmed = battery.trim();
            if let Ok(parsed) = trimmed.parse() {
                return Some((trimmed.to_owned(), parsed));
            }
        }
    }

    None
}

/// Gets battery for a specific battery e.g. BAT0 or BAT1.
fn try_get_battery_index(i: u32) -> Option<String> {
    let output = Command::new("cat")
        .arg(format!("/sys/class/power_supply/BAT{}/capacity", i))
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

struct NiriWindowData {
    title: String,
    app_id: String,
    // everything else is unneeded
}

/// Handles logic for the window display button, communicating with niri to get the appropriate title/icon
async fn update_window_display(label: Label, icon: Image) {
    let subprocess = Subprocess::newv(
        &[
            OsStr::new("niri"),
            OsStr::new("msg"),
            OsStr::new("--json"),
            OsStr::new("event-stream"),
        ],
        SubprocessFlags::STDOUT_PIPE,
    )
    .expect("Failed to start niri event stream");
    let stdout_pipe = subprocess
        .stdout_pipe()
        .expect("Failed to get niri event stream stdout");
    let reader = DataInputStream::new(&stdout_pipe);
    let mut windows: HashMap<i32, NiriWindowData> = HashMap::new();
    let display = Display::default().expect("Cannot connect to display");
    let icon_theme = IconTheme::for_display(&display);
    loop {
        match reader.read_line_utf8_future(glib::Priority::DEFAULT).await {
            Ok(Some(string)) => match json::parse(&string) {
                Ok(parsed) => {
                    if parsed["WindowFocusChanged"].has_key("id") {
                        if let Some(id) = parsed["WindowFocusChanged"]["id"].as_i32() {
                            if let Some(win) = windows.get(&id) {
                                label.set_text(&win.title);
                                icon.set_icon_name(Some(find_icon_for_app(
                                    &icon_theme,
                                    &win.app_id,
                                )));
                                label.set_visible(true);
                                icon.set_visible(true);
                            }
                        } else {
                            // "null" meaning no window focused
                            label.set_visible(false);
                            icon.set_visible(false);
                        }
                    } else if parsed.has_key("WindowsChanged") {
                        let arr = &parsed["WindowsChanged"]["windows"];
                        for val in arr.members() {
                            if let Some((id, data)) = window_json_to_data(val) {
                                if val["is_focused"].as_bool().is_some_and(|b| b) {
                                    label.set_text(&data.title);
                                    icon.set_icon_name(Some(find_icon_for_app(
                                        &icon_theme,
                                        &data.app_id,
                                    )));
                                    label.set_visible(true);
                                    icon.set_visible(true);
                                }
                                windows.insert(id, data);
                            }
                        }
                    } else if let Some(id) = parsed["WindowClosed"]["id"].as_i32() {
                        windows.remove(&id);
                    } else if parsed.has_key("WindowOpenedOrChanged")
                        && let Some((id, data)) =
                            window_json_to_data(&parsed["WindowOpenedOrChanged"]["window"])
                    {
                        if parsed["WindowOpenedOrChanged"]["window"]["is_focused"]
                            .as_bool()
                            .is_some_and(|b| b)
                        {
                            label.set_text(&data.title);
                            icon.set_icon_name(Some(find_icon_for_app(&icon_theme, &data.app_id)));
                            label.set_visible(true);
                            icon.set_visible(true);
                        }
                        windows.insert(id, data);
                    }
                }
                Err(e) => {
                    error!("Failed to parse niri message: {}", e);
                }
            },
            Ok(None) => {
                // EOF reached (process finished writing to stdout)
                error!("Niri stopped sending event stream");
                break;
            }
            Err(err) => {
                error!("Error reading line: {}", err);
                break;
            }
        }
    }
}

/// Parses a window from json and returns a tuple with its id and data.
fn window_json_to_data(window_json: &JsonValue) -> Option<(i32, NiriWindowData)> {
    let id = window_json["id"].as_i32()?;
    let title = window_json["title"].as_str()?.to_owned();
    let app_id = window_json["app_id"].as_str()?.to_owned();
    Some((id, NiriWindowData { app_id, title }))
}

/// Returns a usable icon name for the app with app_id, or a placeholder if none are found.
/// TODO: better searching, if it matters
fn find_icon_for_app<'a>(icon_theme: &IconTheme, app_id: &'a str) -> &'a str {
    if icon_theme.has_icon(app_id) {
        app_id
    } else {
        "pegged-wheels-symbolic"
    }
}

fn add_app_menu_popover(app: &Application, button: &Button) {
    // 1. Create actions and add them to the application scope
    let action_close = gio::SimpleAction::new("close_window", None);
    action_close.connect_activate(|_, _| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("close-window")
            .spawn();
    });
    app.add_action(&action_close);

    let action_max = gio::SimpleAction::new("maximize", None);
    action_max.connect_activate(|_, _| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("maximize-column")
            .spawn();
    });
    app.add_action(&action_max);

    let action_max_edges = gio::SimpleAction::new("maximize_edges", None);
    action_max_edges.connect_activate(move |_, _| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("maximize-window-to-edges")
            .spawn();
    });
    app.add_action(&action_max_edges);

    let menu = gio::Menu::new();
    menu.append(Some("Close"), Some("app.close_window"));
    menu.append(Some("Toggle Maximize"), Some("app.maximize"));
    menu.append(Some("Toggle Maximize to Edges"), Some("app.maximize_edges"));

    let popover = PopoverMenu::from_model(Some(&menu));
    popover.set_parent(button);
    button.connect_clicked(move |_| {
        popover.popup();
    });
}
