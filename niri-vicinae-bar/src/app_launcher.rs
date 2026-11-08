use std::{process::Command, time::Duration};

use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use gtk4::{
    FlowBox, FlowBoxChild, Image, Label, ScrolledWindow, SearchEntry, glib,
    prelude::{BoxExt, EditableExt, GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use libadwaita::{
    Application, ApplicationWindow, CallbackAnimationTarget, Easing, TimedAnimation,
    prelude::{AdwApplicationWindowExt, AnimationExt},
};

pub struct LauncherContext {
    is_open: bool,
    launcher_win: Option<ApplicationWindow>,
}

impl LauncherContext {
    pub fn new() -> Self {
        return LauncherContext {
            is_open: false,
            launcher_win: None,
        };
    }

    pub fn open(&mut self, app: &Application) {
        self.is_open = true;
        let win = ApplicationWindow::builder()
            .application(app)
            .css_classes(["app-launcher"])
            .build();

        // fade in/out
        let start_fade = build_fade_anim(&win, false);

        let locales = get_languages_from_env();
        let desktop_entries = desktop_entries(&locales);

        let entry = SearchEntry::builder()
            .placeholder_text("Search for an app...")
            .hexpand(true)
            .build();

        let apps_box = FlowBox::builder()
            .vexpand(true)
            .selection_mode(gtk4::SelectionMode::None)
            .column_spacing(8)
            .row_spacing(8)
            .build();
        apps_box.connect_child_activated(|_, child| {
            // jank solution since connect_activate is broken
            let name = child.widget_name();
            let mut iter = name.split(' ');
            let mut cmd = Command::new(iter.next().unwrap());
            for arg in iter {
                cmd.arg(arg);
            }
            let _ = cmd.spawn();
            let _ = child.activate_action("app.toggle-launcher", None);
        });

        // Convert desktop entries to buttons
        // async here is a hacky workaround to the lag spike
        // that otherwise happens, makes a natural fade in animation happen
        glib::spawn_future_local(glib::clone!(
            #[strong]
            apps_box,
            async move {
                for entry in desktop_entries {
                    if let Ok(e) = entry.parse_exec() {
                        let name = entry.name(&locales).map(|c| c.to_string()).unwrap_or(
                            entry
                                .generic_name(&locales)
                                .map(|c| c.to_string())
                                .unwrap_or(entry.appid.to_owned()),
                        );
                        let icon = entry.icon().unwrap_or("external-link-symbolic");
                        let btn_box = gtk4::Box::builder()
                            .orientation(gtk4::Orientation::Vertical)
                            .spacing(4)
                            .build();
                        let img = if icon.starts_with('/') || icon.starts_with('~') {
                            Image::from_file(icon)
                        } else {
                            Image::from_icon_name(icon)
                        };
                        img.set_icon_size(gtk4::IconSize::Large);
                        img.set_size_request(64, 64);
                        btn_box.append(&img);
                        let lbl = Label::builder()
                            .label(&name)
                            .ellipsize(gtk4::pango::EllipsizeMode::End)
                            .halign(gtk4::Align::Center)
                            .hexpand(true)
                            .single_line_mode(true)
                            .max_width_chars(15)
                            .build();
                        btn_box.append(&lbl);
                        let btn = FlowBoxChild::builder().child(&btn_box).build();
                        // hacky fix since connect_activate just doesn't trigger ever
                        btn.set_widget_name(
                            &e.iter()
                                .map(|s| s.to_owned())
                                .fold(String::new(), |a, e| format!("{} {}", a, e))
                                .trim(), // removes space at the start
                        );
                        btn.set_tooltip_text(Some(&name));

                        // fade in anim for the btn
                        let btn_target = CallbackAnimationTarget::new(glib::clone!(
                            #[weak]
                            btn,
                            move |v| {
                                btn.set_opacity(v);
                            }
                        ));
                        let fade = TimedAnimation::builder()
                            .value_from(0.0)
                            .value_to(1.0)
                            .widget(&btn)
                            .target(&btn_target)
                            .easing(Easing::EaseInCirc)
                            .duration(250)
                            .build();
                        apps_box.insert(&btn, -1);
                        fade.play();
                        glib::timeout_future(Duration::from_millis(1)).await
                    }
                }
            }
        ));

        let scroll_container = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();
        scroll_container.set_child(Some(&apps_box));

        let outer_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(24)
            .margin_bottom(8)
            .margin_top(8)
            .margin_end(8)
            .margin_start(8)
            .build();
        outer_box.append(&entry);
        outer_box.append(&scroll_container);

        entry.connect_text_notify(glib::clone!(
            #[strong]
            apps_box,
            move |entry| {
                let search_text = entry.text().to_string();
                apps_box.set_filter_func(move |c| {
                    c.tooltip_text()
                        .filter(|t| t.to_lowercase().contains(&search_text.to_lowercase()))
                        .is_some()
                });
            }
        ));

        entry.connect_activate(glib::clone!(
            #[strong]
            apps_box,
            move |entry| {
                // idk how to deal with this, Widget::observe_children is not helping
                // also can't seem to tell if the widget is shown or not (is_visible always true)
                let mut idx = 0;
                let txt = entry.text().to_lowercase();
                while let Some(c) = apps_box.child_at_index(idx) {
                    if c.tooltip_text()
                        .filter(|t| t.to_lowercase().contains(&txt))
                        .is_some()
                    {
                        c.activate();
                    }
                    idx += 1;
                }
            }
        ));

        win.set_content(Some(&outer_box));

        // Layer shell to take up rest of screen
        win.init_layer_shell();
        win.set_layer(gtk4_layer_shell::Layer::Top);
        win.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        win.set_anchor(gtk4_layer_shell::Edge::Top, true);
        win.set_anchor(gtk4_layer_shell::Edge::Left, true);
        win.set_anchor(gtk4_layer_shell::Edge::Right, true);
        win.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
        win.show();
        start_fade.play();
        self.launcher_win = Some(win);
    }

    pub fn close(&mut self) {
        if let Some(win) = &self.launcher_win {
            let fade = build_fade_anim(win, true);
            fade.play();
            fade.connect_done(glib::clone!(
                #[strong]
                win,
                move |_| {
                    win.close();
                }
            ));
            self.is_open = false;
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

fn build_fade_anim(win: &ApplicationWindow, reversed: bool) -> TimedAnimation {
    // fade in/out
    let opacity_target = CallbackAnimationTarget::new(glib::clone!(
        #[weak]
        win,
        move |v| {
            win.set_opacity(v);
        }
    ));
    TimedAnimation::builder()
        .value_from(if reversed { 1.0 } else { 0.0 })
        .value_to(if reversed { 0.0 } else { 1.0 })
        .widget(win)
        .target(&opacity_target)
        .easing(Easing::EaseOutCirc)
        .duration(300)
        .build()
}
