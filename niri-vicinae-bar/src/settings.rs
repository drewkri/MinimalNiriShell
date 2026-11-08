use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use dirs::config_dir;
use freedesktop_desktop_entry::{DesktopEntry, desktop_entries, get_languages_from_env};
use gtk4::{
    Button, Frame, Image, Label, ListBox, ScrolledWindow, SearchEntry,
    glib::{
        self,
        object::{Cast, ObjectExt},
    },
    prelude::{BoxExt, ButtonExt, EditableExt, FrameExt, WidgetExt},
};
use libadwaita::{
    ActionRow, Application, ApplicationWindow, ButtonRow, Dialog, EntryRow, HeaderBar,
    PreferencesGroup, PreferencesPage, SwitchRow, ToolbarView,
    prelude::{
        ActionRowExt, AdwApplicationWindowExt, AdwDialogExt, EntryRowExt, PreferencesGroupExt,
        PreferencesPageExt,
    },
};
use serde::{Deserialize, Serialize};

use crate::APP_ID;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub shortcuts: Vec<PathBuf>,
    pub inverse_workspace_scroll: bool,
    pub notifications_cmd: String,
    pub network_settings_cmd: String,
    pub audio_settings_cmd: String,
    pub time_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shortcuts: Vec::new(),
            inverse_workspace_scroll: false,
            notifications_cmd: String::from("swaync-client -t"),
            network_settings_cmd: String::from("nm-sidebar --toggle"),
            audio_settings_cmd: String::from("alacritty -e wiremix"),
            time_format: String::from("%H:%M"),
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

    fn config_path() -> PathBuf {
        let mut p = config_dir().expect("User has no config dir");
        p.push(APP_ID);
        if !p.exists() {
            fs::create_dir_all(&p).expect("Cannot create config dir");
        }
        p.push("config.toml");
        p
    }
}

pub fn open_settings_window(app: &Application) {
    let cfg = Config::try_load();
    let cfg = Arc::new(RwLock::new(cfg));
    let win = ApplicationWindow::builder()
        .title("Niri Touch HUD Settings")
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

    let shortcuts_group = PreferencesGroup::builder()
        .title("Add/Remove App Shortcuts")
        .hexpand(true)
        .build();
    populate_shortcuts_group(&shortcuts_group, cfg.clone());
    page.add(&shortcuts_group);

    let cfg_read = cfg.read().unwrap();

    let commands_group = PreferencesGroup::builder()
        .title("Change Button Commands")
        .build();
    page.add(&commands_group);

    let audio_row = EntryRow::builder()
        .title("Audio Settings Command")
        .text(&cfg_read.audio_settings_cmd)
        .show_apply_button(true)
        .build();
    let cfg_clone = cfg.clone();
    audio_row.connect_apply(move |row| {
        let mut bind = cfg_clone.write().expect("Broken lock");
        bind.audio_settings_cmd = row.text().into();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    commands_group.add(&audio_row);
    let wifi_row = EntryRow::builder()
        .title("Network Settings Command")
        .text(&cfg_read.network_settings_cmd)
        .show_apply_button(true)
        .build();
    let cfg_clone = cfg.clone();
    wifi_row.connect_apply(move |row| {
        let mut bind = cfg_clone.write().expect("Broken lock");
        bind.network_settings_cmd = row.text().into();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    commands_group.add(&wifi_row);
    let notif_row = EntryRow::builder()
        .title("Notification Panel Command")
        .text(&cfg_read.notifications_cmd)
        .show_apply_button(true)
        .build();
    let cfg_clone = cfg.clone();
    notif_row.connect_apply(move |row| {
        let mut bind = cfg_clone.write().expect("Broken lock");
        bind.notifications_cmd = row.text().into();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    commands_group.add(&notif_row);

    let misc_group = PreferencesGroup::builder().title("Miscellaneous").build();
    let scroll_dir_row = SwitchRow::builder()
        .title("Inverted Workspace Scrolling")
        .subtitle("If true, swiping down on the workspaces widget moves to the workspace above.")
        .active(cfg_read.inverse_workspace_scroll)
        .build();
    let scroll_cfg_clone = cfg.clone();
    scroll_dir_row.connect_notify(Some("active"), move |row, _| {
        let mut bind = scroll_cfg_clone.write().expect("Broken lock");
        bind.inverse_workspace_scroll = row.is_active();
        bind.save();
        let _ = row.activate_action("app.reload-config", None);
    });
    misc_group.add(&scroll_dir_row);
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

fn populate_shortcuts_group(shortcuts_group: &PreferencesGroup, cfg: Arc<RwLock<Config>>) {
    let locales = get_languages_from_env();
    let cfg_read = cfg.read().expect("Broken lock");
    for s in &cfg_read.shortcuts {
        let row;
        let del_btn = Button::builder().icon_name("user-trash-symbolic").build();
        let cfg_clone = cfg.clone();
        let s_clone = s.clone();
        del_btn.connect_clicked(move |btn| {
            let mut bind = cfg_clone.try_write().expect("Broken lock");
            bind.shortcuts.retain(|e| *e != s_clone);
            bind.save();
            let _ = btn.activate_action("app.reload-config", None);
        });
        if let Ok(entry) = DesktopEntry::from_path(s, Some(&locales)) {
            row = ActionRow::builder()
                .title(
                    entry
                        .name(&locales)
                        .map(|s| s.to_string())
                        .or(entry.generic_name(&locales).map(|c| c.to_string()))
                        .unwrap_or(entry.appid.to_owned()),
                )
                .subtitle(s.to_str().unwrap_or("Unknown File Path"))
                .build();
        } else {
            row = ActionRow::builder()
                .title("Corrupted Shortcut")
                .subtitle("This app's desktop file is missing or broken.")
                .build();
        }
        row.add_suffix(&del_btn);
        shortcuts_group.add(&row);
    }

    let add_btn = ButtonRow::builder()
        .title("Add new shortcut...")
        .start_icon_name("plus-symbolic")
        .hexpand(true)
        .build();
    let cfg_clone = cfg.clone();
    add_btn.connect_activated(move |btn| {
        let dialog = Dialog::builder().title("Add a New Shortcut").build();
        let tool_view = ToolbarView::builder().build();
        let header = HeaderBar::builder().build();
        tool_view.add_top_bar(&header);
        dialog.set_child(Some(&tool_view));

        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(16)
            .margin_start(16)
            .margin_end(16)
            .margin_bottom(16)
            .margin_top(16)
            .build();
        let lbl = Label::builder()
            .label("Choose an app to add a shortcut to...")
            .halign(gtk4::Align::Start)
            .build();
        content_box.append(&lbl);
        let frame = Frame::builder()
            .hexpand(true)
            .vexpand(true)
            .width_request(450)
            .height_request(650)
            .build();
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();
        frame.set_child(Some(&scroll));
        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        scroll.set_child(Some(&list_box));

        // get apps, populate list box
        // will likely cause some lag mostly due to loading images
        // but it's the settings menu idc
        let apps = desktop_entries(&locales);
        for entry in apps {
            let name = entry
                .name(&locales)
                .map(|s| s.to_string())
                .or(entry.generic_name(&locales).map(|s| s.to_string()))
                .unwrap_or(entry.appid.clone());
            let row = ActionRow::builder()
                .title(&name)
                .subtitle(entry.path.to_str().unwrap_or("Unknown File Path"))
                .name(name)
                .build();
            let icon = entry.icon().unwrap_or("external-link-symbolic");
            let img = if icon.starts_with('/') || icon.starts_with('~') {
                Image::from_file(icon)
            } else {
                Image::from_icon_name(icon)
            };
            img.set_icon_size(gtk4::IconSize::Normal);
            row.add_prefix(&img);
            list_box.append(&row);
        }

        let search = SearchEntry::builder()
            .placeholder_text("Search for an app...")
            .build();
        search.connect_text_notify(glib::clone!(
            #[weak]
            list_box,
            move |entry| {
                let text = entry.text().to_string();
                list_box.set_filter_func(move |e| {
                    e.widget_name()
                        .to_lowercase()
                        .contains(&text.to_lowercase())
                });
            }
        ));

        content_box.append(&search);
        content_box.append(&frame);

        let add_btn = Button::builder()
            .label("Add a shortcut for this app...")
            // .icon_name("plus-symbolic")
            .css_classes(["suggested-action"])
            .build();
        let cfg_clone = cfg_clone.clone();
        add_btn.connect_clicked(glib::clone!(
            #[weak]
            list_box,
            #[weak]
            dialog,
            move |btn| {
                let selected = list_box.selected_row();
                if let Some(row) = selected {
                    // SHOULD always succeed
                    if let Ok(action_row) = row.downcast::<ActionRow>() {
                        let path = action_row.subtitle().unwrap();
                        let mut bind = cfg_clone.write().expect("Broken lock");
                        bind.shortcuts.push(PathBuf::from(path));
                        bind.save();
                        dialog.close();
                        let _ = btn.activate_action("app.reload-config", None);
                    }
                }
            }
        ));
        content_box.append(&add_btn);

        tool_view.set_content(Some(&content_box));
        dialog.present(Some(btn));
    });
    shortcuts_group.add(&add_btn);
}
