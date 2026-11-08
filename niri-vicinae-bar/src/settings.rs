use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use dirs::config_dir;
use gtk4::{
    glib::object::ObjectExt,
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    Application, ApplicationWindow, EntryRow, HeaderBar, PreferencesGroup, PreferencesPage,
    SwitchRow, ToolbarView,
    prelude::{AdwApplicationWindowExt, EntryRowExt, PreferencesGroupExt, PreferencesPageExt},
};
use serde::{Deserialize, Serialize};

use crate::APP_ID;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub time_format: String,
    pub battery: bool,
    pub bar_on_top: bool,
    pub show_keyboard_button: bool,

    pub keyboard_button_command: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            battery: true,
            time_format: "%I:%M %p".to_owned(),
            bar_on_top: true,
            show_keyboard_button: true,
            keyboard_button_command: "wvkbd".to_owned(),
        }
    }
}

impl Config {
    pub fn try_load() -> Self {
        let p = Self::config_path();
        let opt = fs::read_to_string(&p)
            .ok()
            .and_then(|s| toml::from_str::<Config>(&s).ok());
        if let Some(cfg) = opt {
            cfg
        } else {
            let cfg = Config::default();
            cfg.save();
            cfg
        }
    }

    pub fn save(&self) {
        let p = Self::config_path();
        fs::write(p, toml::to_string(self).unwrap()).expect("Can't write to config file.");
    }

    pub fn config_path() -> PathBuf {
        let mut p = config_dir().expect("User has no config dir");
        p.push(APP_ID);
        if !p.exists() {
            fs::create_dir_all(&p).expect("Cannot create config dir");
        }
        p.push("config.toml");
        p
    }
}

// messy but it works
// mostly copied and pasted code to recreate the same components
pub fn open_settings_window(app: &Application) {
    let cfg = Config::try_load();
    let cfg = Arc::new(RwLock::new(cfg));
    let win = ApplicationWindow::builder()
        .title("Niri Vicinae Bar Settings")
        .application(app)
        .build();
    let header = HeaderBar::builder().build();
    let tool_view = ToolbarView::builder().hexpand(true).vexpand(true).build();
    let page = PreferencesPage::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    tool_view.set_content(Some(&page));
    tool_view.add_top_bar(&header);
    win.set_content(Some(&tool_view));

    let cfg_read = cfg.read().unwrap();

    let commands_group = PreferencesGroup::builder()
        .title("Change Button Commands")
        .build();
    let keyb_cmd_row = EntryRow::builder()
        .title("Keyboard Button Command")
        .text(&cfg_read.keyboard_button_command)
        .show_apply_button(true)
        .build();
    let keyb_cfg_clone = cfg.clone();
    keyb_cmd_row.connect_apply(move |row| {
        let mut bind = keyb_cfg_clone.write().expect("Broken lock");
        bind.keyboard_button_command = row.text().into();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    commands_group.add(&keyb_cmd_row);
    page.add(&commands_group);

    let misc_group = PreferencesGroup::builder().title("Miscellaneous").build();
    let battery_row = SwitchRow::builder()
        .title("Show Battery Indicator")
        .subtitle(
            "Enabling this will show a battery indicator on the bar. Not recommended for desktops.",
        )
        .active(cfg_read.battery)
        .build();
    let battery_cfg_clone = cfg.clone();
    battery_row.connect_notify(Some("active"), move |row, _| {
        let mut bind = battery_cfg_clone.write().expect("Broken lock");
        bind.battery = row.is_active();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    misc_group.add(&battery_row);
    let bar_row = SwitchRow::builder()
        .title("Bar on Top")
        .subtitle(
            "Enabling this will place the bar on the top of the screen, while disabling will place it on the bottom.",
        )
        .active(cfg_read.bar_on_top)
        .build();
    let bar_cfg_clone = cfg.clone();
    bar_row.connect_notify(Some("active"), move |row, _| {
        let mut bind = bar_cfg_clone.write().expect("Broken lock");
        bind.bar_on_top = row.is_active();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    misc_group.add(&bar_row);
    let keyb_row = SwitchRow::builder()
        .title("Show Keyboard Button")
        .subtitle("Adds a button to the bar to open an on-screen keyboard. You may need to set this button's command above.")
        .active(cfg_read.show_keyboard_button)
        .build();
    let keyb_cfg_clone = cfg.clone();
    keyb_row.connect_notify(Some("active"), move |row, _| {
        let mut bind = keyb_cfg_clone.write().expect("Broken lock");
        bind.show_keyboard_button = row.is_active();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    misc_group.add(&keyb_row);
    let time_fmt_row = EntryRow::builder()
        .title("Time Display Format (uses chrono::format::strftime)")
        .text(&cfg_read.time_format)
        .show_apply_button(true)
        .build();
    let cfg_clone = cfg.clone();
    time_fmt_row.connect_apply(move |row| {
        let mut bind = cfg_clone.write().expect("Broken lock");
        bind.time_format = row.text().into();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    misc_group.add(&time_fmt_row);
    page.add(&misc_group);

    win.show();
}
