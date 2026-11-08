use clap::Parser;
use freedesktop_desktop_entry::{DesktopEntry, get_languages_from_env};
use std::{
    cell::RefCell,
    collections::HashMap,
    process::Command,
    sync::{Arc, RwLock},
};
use zbus::blocking;

use chrono::{Local, Timelike};
use gtk4::{
    CssProvider, GestureSwipe, IconTheme, Image, Widget,
    gdk::Display,
    gio::{
        self, ActionEntry,
        prelude::{ActionMapExtManual, ApplicationExt, ApplicationExtManual},
        resources_register_include,
    },
    glib::{self, ExitCode, object::IsA},
    prelude::{ActionableExt, BoxExt, ButtonExt, GtkApplicationExt, GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use libadwaita::{Application, prelude::AdwApplicationWindowExt};
use niri_ipc::{Event, Workspace};

use crate::{
    app_launcher::LauncherContext,
    dbus_actions::ActionsProxyBlocking,
    settings::{Config, open_settings_window},
};

const APP_ID: &str = "io.github.DrewCodesBadly.NiriTouchHud";
const CSS_STRING: &str = include_str!("style.css");

mod app_launcher;
mod dbus_actions;
mod settings;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long = "toggle-launcher")]
    toggle_launcher: bool,
}

// awful code quality here because this is a slop app I just need to work for myself
// sorry to anyone reading this

fn main() -> glib::ExitCode {
    let args = Args::parse();
    if args.toggle_launcher {
        let conn = blocking::Connection::session().expect("Failed to connect to dbus user session");
        let proxy = ActionsProxyBlocking::new(&conn).unwrap();
        proxy
            .activate("toggle-launcher", &[], HashMap::new())
            .unwrap();
        return ExitCode::SUCCESS;
    }
    let _ = resources_register_include!("niri-touch-hud.gresource");
    let app = Application::builder().application_id(APP_ID).build();
    // startup tasks
    app.connect_startup(|_| {
        let display = Display::default().expect("No display connected.");

        // Handle loading CSS
        let provider = CssProvider::new();
        provider.load_from_data(CSS_STRING);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // load icons
        let theme = IconTheme::for_display(&display);
        theme.add_resource_path("/hud/icons");
    });
    app.connect_activate(activate);

    app.run();

    glib::ExitCode::SUCCESS
}

fn activate(app: &Application) {
    let cfg = Config::try_load();
    // Create two windows: Left side and right side
    let left_win = build_left_window(app, &cfg);
    let bottom_win = build_bottom_window(app, &cfg);
    bottom_win.set_size_request(1, 1);
    left_win.set_size_request(1, 1);
    // Bottom first, as it has the corner contents
    bottom_win.show();
    left_win.show();

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
                let left_win = build_left_window(&app, &cfg);
                let bottom_win = build_bottom_window(&app, &cfg);
                bottom_win.set_size_request(1, 1);
                left_win.set_size_request(1, 1);
                // Bottom first, as it has the corner contents
                bottom_win.show();
                left_win.show();
            }
        ))
        .build()]);
}

// not currently built to handle multiple monitors
fn rebuild_workspace_widget(list: &Vec<Workspace>) -> impl IsA<Widget> {
    let b = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .build();

    for w in list {
        let name = if w.is_focused {
            "circle-filled-symbolic"
        } else {
            "circle-outline-thick-symbolic"
        };
        let i = Image::builder().icon_name(name).build();
        b.append(&i);
    }

    b
}

fn build_left_window(app: &Application, cfg: &Config) -> libadwaita::ApplicationWindow {
    let window = libadwaita::ApplicationWindow::new(app);

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Top);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.auto_exclusive_zone_enable();

    let left_box = gtk4::CenterBox::builder()
        .orientation(gtk4::Orientation::Vertical)
        .vexpand(true)
        .margin_start(4)
        .margin_end(4)
        .margin_top(6)
        .margin_bottom(6)
        .build();
    window.set_content(Some(&left_box));

    let overview_btn = gtk4::Button::builder()
        .icon_name("tabs-stack-symbolic")
        .valign(gtk4::Align::Start)
        .halign(gtk4::Align::Center)
        .vexpand(false)
        .css_classes(["circular", "opaque"])
        .build();
    overview_btn.connect_clicked(|_| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("open-overview")
            .spawn();
    });

    let close_btn = gtk4::Button::builder()
        .icon_name("cross-large-symbolic")
        .halign(gtk4::Align::Center)
        .vexpand(false)
        .css_classes(["circular"])
        .build();

    overview_btn.connect_clicked(|_| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("open-overview")
            .spawn();
    });

    close_btn.connect_clicked(|_| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("close-window")
            .spawn();
    });

    let buttons_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .vexpand(false)
        .spacing(32)
        .build();
    buttons_box.append(&overview_btn);
    buttons_box.append(&close_btn);

    let workspaces_box = gtk4::CenterBox::builder()
        .orientation(gtk4::Orientation::Vertical)
        .vexpand(true)
        .build();

    let swipe_controller = GestureSwipe::new();
    let inverted_scroll = cfg.inverse_workspace_scroll;
    swipe_controller.connect_swipe(move |_, _, vy| {
        if (inverted_scroll && vy < 0.0) || (!inverted_scroll && vy > 0.0) {
            let _ = Command::new("niri")
                .arg("msg")
                .arg("action")
                .arg("focus-workspace-down")
                .spawn();
        } else {
            let _ = Command::new("niri")
                .arg("msg")
                .arg("action")
                .arg("focus-workspace-up")
                .spawn();
        }
    });
    workspaces_box.add_controller(swipe_controller);

    glib::spawn_future_local(glib::clone!(
        #[weak]
        workspaces_box,
        async move {
            // unwraps are used here since this whole thing won't work if niri isn't running anyway.
            let mut niri_socket = niri_ipc::socket::Socket::connect().unwrap();
            let _ = niri_socket.send(niri_ipc::Request::EventStream);
            let read_events = niri_socket.read_events();
            let lock = Arc::new(RwLock::new(read_events));

            let mut list = Vec::new();

            loop {
                let lock_clone = lock.clone();
                let response = gio::spawn_blocking(move || lock_clone.write().unwrap()())
                    .await
                    .unwrap();
                if let Ok(event) = response {
                    match event {
                        Event::WorkspacesChanged { workspaces } => {
                            list = workspaces;
                            list.sort_by(|w1, w2| w1.idx.cmp(&w2.idx));
                            let b = rebuild_workspace_widget(&list);
                            workspaces_box.set_center_widget(Some(&b));
                        }
                        Event::WorkspaceActivated { id, focused: _ } => {
                            for w in &mut list {
                                w.is_focused = w.id == id;
                            }
                            let b = rebuild_workspace_widget(&list);
                            workspaces_box.set_center_widget(Some(&b));
                        }
                        _ => {}
                    }
                }
            }
        }
    ));

    let shortcuts_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .vexpand(false)
        .spacing(8)
        .build();
    let settings_btn = gtk4::Button::builder()
        .icon_name("wrench-wide-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    shortcuts_box.append(&settings_btn);
    let locales = get_languages_from_env();
    for shortcut in cfg.shortcuts.iter().rev() {
        if let Ok(entry) = DesktopEntry::from_path(shortcut, Some(&locales)) {
            let icon = entry.icon().unwrap_or("external-link-symbolic");
            let img = if icon.starts_with('/') || icon.starts_with('~') {
                Image::from_file(icon)
            } else {
                Image::from_icon_name(icon)
            };
            let btn = gtk4::Button::builder()
                .css_classes(["circular"])
                .child(&img)
                .build();
            // should always be true but in case not the shortcut won't show up instead of crashing
            if let Ok(exec) = entry.parse_exec() {
                btn.connect_clicked(move |_| {
                    let mut iter = exec.iter();
                    let mut cmd = Command::new(iter.next().unwrap());
                    for arg in iter {
                        cmd.arg(arg);
                    }
                    let _ = cmd.spawn();
                });
            }
            shortcuts_box.append(&btn);
        }
    }

    settings_btn.connect_clicked(glib::clone!(
        #[weak]
        app,
        move |_| {
            open_settings_window(&app);
        }
    ));

    left_box.set_start_widget(Some(&buttons_box));
    left_box.set_center_widget(Some(&workspaces_box));
    left_box.set_end_widget(Some(&shortcuts_box));

    window
}

fn build_bottom_window(app: &Application, cfg: &Config) -> libadwaita::ApplicationWindow {
    let window = libadwaita::ApplicationWindow::new(app);

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Top);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.auto_exclusive_zone_enable();

    let bottom_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(true)
        .margin_bottom(4)
        .margin_top(4)
        .margin_end(6)
        .margin_start(6)
        .build();
    window.set_content(Some(&bottom_box));

    // Launcher
    let launcher_context = LauncherContext::new();
    let cell = RefCell::new(launcher_context);
    let launcher_button = gtk4::Button::builder()
        .icon_name("menu-large-symbolic")
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Start)
        .hexpand(false)
        .css_classes(["circular", "opaque"])
        .build();
    let toggle_launcher_act = ActionEntry::builder("toggle-launcher")
        .activate(glib::clone!(
            #[weak]
            app,
            move |_, _, _| {
                let mut bind = cell.borrow_mut();
                if bind.is_open() {
                    bind.close();
                } else {
                    bind.open(&app);
                }
            }
        ))
        .build();
    app.add_action_entries([toggle_launcher_act]);
    launcher_button.set_action_name(Some("app.toggle-launcher"));

    let misc_widgets_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(true)
        .halign(gtk4::Align::Center)
        .spacing(32)
        // .margin_bottom(4)
        // .margin_end(4)
        // .margin_start(4)
        // .margin_top(4)
        .build();
    let niri_controls_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(true)
        .halign(gtk4::Align::Center)
        .spacing(32)
        // .margin_bottom(4)
        // .margin_end(4)
        // .margin_start(4)
        // .margin_top(4)
        .build();
    let notif_button = gtk4::Button::builder()
        .icon_name("bell-outline-symbolic")
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::End)
        .hexpand(false)
        .css_classes(["circular", "opaque"])
        .build();

    // Misc Widgets
    let wifi_btn = gtk4::Button::builder()
        .icon_name("radiowaves-1-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    let perf_btn = gtk4::Button::builder()
        .icon_name("speedometer-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    let audio_btn = gtk4::Button::builder()
        .icon_name("speakers-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    let audio_cmd = cfg.audio_settings_cmd.clone();
    audio_btn.connect_clicked(move |_| {
        parse_and_run_command(&audio_cmd);
    });
    let wifi_cmd = cfg.network_settings_cmd.clone();
    wifi_btn.connect_clicked(move |_| {
        parse_and_run_command(&wifi_cmd);
    });
    misc_widgets_box.append(&wifi_btn);
    misc_widgets_box.append(&perf_btn);
    misc_widgets_box.append(&audio_btn);

    // Date/time
    let time_lbl = gtk4::Label::builder()
        .label("")
        .css_classes(["numeric", "title-2"])
        // .css_classes(["numeric"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .hexpand(true)
        .build();

    let fmt_str_clone = cfg.time_format.clone();
    glib::spawn_future_local(glib::clone!(
        #[weak]
        time_lbl,
        async move {
            let mut prev_now_minutes = 67; // arbitrary impossible number :3
            loop {
                let now = Local::now();
                let minute = now.minute();
                if minute != prev_now_minutes {
                    // refresh everything
                    prev_now_minutes = minute;
                    time_lbl.set_text(&now.format(&fmt_str_clone).to_string());
                    time_lbl.set_tooltip_text(Some(&now.format("%A, %B%e").to_string()));
                }
                glib::timeout_future_seconds(1).await;
            }
        }
    ));

    // Niri controls
    let left_btn = gtk4::Button::builder()
        .icon_name("left-large-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    left_btn.connect_clicked(|_| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("focus-column-left")
            .spawn();
    });
    let fullscreen_btn = gtk4::Button::builder()
        .icon_name("arrows-pointing-outward-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    fullscreen_btn.connect_clicked(|_| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("maximize-column")
            .spawn();
    });
    let right_btn = gtk4::Button::builder()
        .icon_name("right-large-symbolic")
        .valign(gtk4::Align::Center)
        .hexpand(false)
        .css_classes(["circular"])
        .build();
    right_btn.connect_clicked(|_| {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("focus-column-right")
            .spawn();
    });
    niri_controls_box.append(&left_btn);
    niri_controls_box.append(&fullscreen_btn);
    niri_controls_box.append(&right_btn);

    let notif_cmd = cfg.notifications_cmd.clone();
    notif_button.connect_clicked(move |_| {
        parse_and_run_command(&notif_cmd);
    });

    bottom_box.append(&launcher_button);
    bottom_box.append(&misc_widgets_box);
    bottom_box.append(&time_lbl);
    bottom_box.append(&niri_controls_box);
    bottom_box.append(&notif_button);

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
