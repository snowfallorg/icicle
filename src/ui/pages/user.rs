use crate::ui::window::{AppMsg, UserConfig};
use adw::prelude::*;
use gettextrs::gettext;
use log::{debug, trace};
use relm4::*;

#[tracker::track]
pub struct UserModel {
    name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    confirm_password: Option<String>,
    hostname: Option<String>,
    root_password: Option<String>,
    confirm_root_password: Option<String>,
    username_row: adw::EntryRow,
    confirm_password_row: adw::PasswordEntryRow,
    confirm_root_password_row: adw::PasswordEntryRow,
    hostnamerow: adw::EntryRow,
    showhostname: bool,
    showrootpassword: bool,
    autologin: bool,
}

#[derive(Debug)]
pub enum UserMsg {
    SetConfig(bool, bool, String),
    NameChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    ConfirmPasswordChanged(String),
    HostnameChanged(String),
    RootPasswordChanged(String),
    ConfirmRootPasswordChanged(String),
    SetPasswordStyle,
    SetRootPasswordStyle,
    SetAutoLogin(bool),
    CheckSelected,
}

#[relm4::component(pub)]
impl SimpleComponent for UserModel {
    type Init = ();
    type Input = UserMsg;
    type Output = AppMsg;

    view! {
        gtk::ScrolledWindow {
            adw::Clamp {
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,
                    set_margin_all: 20,
                    adw::Avatar {
                        set_size: 144,
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &gettext("Create a new user"),
                        add_css_class: "title-1"
                    },
                    gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Name"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::NameChanged(entry.text().to_string()));
                            }
                        },
                        #[local_ref]
                        username_row -> adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Username"),
                            connect_text_notify => move |entry| {
                                let mut corrected = String::new();
                                for c in entry.text().chars() {
                                    if c.is_ascii_lowercase() || c.is_ascii_digit() {
                                        corrected.push(c);
                                    }
                                }

                                while let Some(first) = corrected.chars().next() {
                                    if first.is_ascii_digit() {
                                        if let Some(c) = corrected.get(1..) {
                                            corrected = c.to_string();
                                        } else {
                                            corrected = String::new();
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }

                                if entry.text() != corrected {
                                    entry.set_text(&corrected);
                                }
                            },
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::UsernameChanged(entry.text().to_string()));
                            },
                        },
                        adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::PasswordChanged(entry.text().to_string()));
                            }
                        },
                        #[local_ref]
                        confirm_password_row -> adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Confirm Password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::ConfirmPasswordChanged(entry.text().to_string()));
                            }
                        },
                        adw::ActionRow {
                            #[watch]
                            set_title: &gettext("Log in automatically"),
                            set_activatable: true,
                            connect_activated[autoswitch] => move |_| {
                                autoswitch.activate();
                            },
                            #[name(autoswitch)]
                            add_suffix = &gtk::Switch {
                                set_valign: gtk::Align::Center,
                                connect_state_set[sender] => move |_, state| {
                                    sender.input(UserMsg::SetAutoLogin(state));
                                    glib::Propagation::Proceed
                                }
                            }
                        }
                    },
                    gtk::ListBox {
                        #[watch]
                        set_visible: model.showhostname,
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        #[local_ref]
                        hostnamerow -> adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Hostname"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::HostnameChanged(entry.text().to_string()));
                            },
                            connect_text_notify => move |entry| {
                                let mut corrected = String::new();
                                for c in entry.text().chars() {
                                    if c.is_ascii_alphanumeric() || c.is_ascii_digit() {
                                        corrected.push(c);
                                    }
                                }

                                if entry.text() != corrected {
                                    entry.set_text(&corrected);
                                }
                            }
                        }
                    },
                    gtk::ListBox {
                        #[watch]
                        set_visible: model.showrootpassword,
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Root password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::RootPasswordChanged(entry.text().to_string()));
                            }
                        },
                        #[local_ref]
                        confirm_root_password_row -> adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Confirm root password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::ConfirmRootPasswordChanged(entry.text().to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = UserModel {
            name: None,
            username: None,
            password: None,
            confirm_password: None,
            hostname: None,
            root_password: None,
            confirm_root_password: None,
            username_row: adw::EntryRow::new(),
            hostnamerow: adw::EntryRow::new(),
            confirm_password_row: adw::PasswordEntryRow::new(),
            confirm_root_password_row: adw::PasswordEntryRow::new(),
            showhostname: false,
            showrootpassword: false,
            autologin: false,
            tracker: 0,
        };
        let username_row = &model.username_row;
        let confirm_password_row = &model.confirm_password_row;
        let confirm_root_password_row = &model.confirm_root_password_row;
        let hostnamerow = &model.hostnamerow;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            UserMsg::SetConfig(root, showhostname, hostname) => {
                self.showrootpassword = root;
                self.showhostname = showhostname;
                self.hostname = Some(hostname.to_string());
                self.hostnamerow.set_text(&hostname);
            }
            UserMsg::NameChanged(name) => {
                // Replace any non a-z characters with an ""
                let suggested_username = name
                    .to_ascii_lowercase()
                    .replace(' ', "")
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .collect::<String>();

                if let Some(username) = &self.username {
                    if let Some(oldname) = &self.name {
                        if username.eq(&oldname
                            .to_ascii_lowercase()
                            .replace(' ', "")
                            .chars()
                            .filter(|c| c.is_ascii_alphanumeric())
                            .collect::<String>())
                        {
                            self.username_row.set_text(&suggested_username);
                        }
                    }
                } else {
                    self.username_row.set_text(&suggested_username);
                }
                self.name = if name.is_empty() { None } else { Some(name) };
                sender.input(UserMsg::CheckSelected);
            }
            UserMsg::UsernameChanged(username) => {
                self.username = if username.is_empty() {
                    None
                } else {
                    Some(username)
                };
                sender.input(UserMsg::CheckSelected);
            }
            UserMsg::PasswordChanged(password) => {
                self.password = if password.is_empty() {
                    None
                } else {
                    Some(password)
                };
                sender.input(UserMsg::SetPasswordStyle);
            }
            UserMsg::ConfirmPasswordChanged(confirm_password) => {
                self.confirm_password = if confirm_password.is_empty() {
                    None
                } else {
                    Some(confirm_password)
                };
                sender.input(UserMsg::SetPasswordStyle);
            }
            UserMsg::SetPasswordStyle => {
                if self.password == self.confirm_password {
                    if self.password.is_some() {
                        self.confirm_password_row.add_css_class("success");
                        self.confirm_password_row.remove_css_class("error");
                    } else {
                        self.confirm_password_row.remove_css_class("success");
                        self.confirm_password_row.remove_css_class("error");
                    }
                } else {
                    self.confirm_password_row.add_css_class("error");
                    self.confirm_password_row.remove_css_class("success");
                }
                sender.input(UserMsg::CheckSelected);
            }
            UserMsg::HostnameChanged(hostname) => {
                if let Some(current) = &self.hostname {
                    if &hostname == current {
                        debug!("Hostname changed to the same value, ignoring");
                        return;
                    }
                }

                self.hostname = if hostname.is_empty() {
                    None
                } else {
                    Some(hostname)
                };
                sender.input(UserMsg::CheckSelected);
            }
            UserMsg::RootPasswordChanged(root_password) => {
                self.root_password = if root_password.is_empty() {
                    None
                } else {
                    Some(root_password)
                };
                sender.input(UserMsg::SetRootPasswordStyle);
            }
            UserMsg::ConfirmRootPasswordChanged(confirm_root_password) => {
                self.confirm_root_password = if confirm_root_password.is_empty() {
                    None
                } else {
                    Some(confirm_root_password)
                };
                sender.input(UserMsg::SetRootPasswordStyle);
            }
            UserMsg::SetRootPasswordStyle => {
                if self.root_password == self.confirm_root_password {
                    if self.root_password.is_some() {
                        self.confirm_root_password_row.add_css_class("success");
                        self.confirm_root_password_row.remove_css_class("error");
                    } else {
                        self.confirm_root_password_row.remove_css_class("success");
                        self.confirm_root_password_row.remove_css_class("error");
                    }
                } else {
                    self.confirm_root_password_row.add_css_class("error");
                    self.confirm_root_password_row.remove_css_class("success");
                }
                sender.input(UserMsg::CheckSelected);
            }
            UserMsg::SetAutoLogin(autologin) => {
                self.autologin = autologin;
                sender.input(UserMsg::CheckSelected);
            }
            UserMsg::CheckSelected => {
                let cangoforward = self.name.is_some()
                    && self.username.is_some()
                    && self.password.is_some()
                    && self.confirm_password.is_some()
                    && self.password == self.confirm_password
                    && self.hostname.is_some()
                    && self.root_password == self.confirm_root_password;
                trace!("UserMsg::CheckSelected {}", cangoforward);

                if cangoforward {
                    if let (
                        Some(name),
                        Some(username),
                        Some(password),
                        Some(_confirm_password),
                        Some(hostname),
                    ) = (
                        &self.name,
                        &self.username,
                        &self.password,
                        &self.confirm_password,
                        &self.hostname,
                    ) {
                        let _ = sender.output(AppMsg::SetUserConfig(Some(UserConfig {
                            name: name.to_string(),
                            username: username.to_string(),
                            password: password.to_string(),
                            hostname: hostname.to_string(),
                            rootpassword: self.root_password.clone(),
                            autologin: self.autologin,
                        })));
                    }
                }

                let _ = sender.output(AppMsg::SetCanGoForward(cangoforward));
            }
        }
    }
}
