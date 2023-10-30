use super::partitions::{CustomPartition, PartitionSchema};
use crate::ui::window::{AppMsg, UserConfig};
use adw::prelude::*;
use gettextrs::gettext;
use gnome_desktop::{self, XkbInfo, XkbInfoExt};
use log::debug;
use relm4::{factory::*, *};

#[tracker::track]
pub struct SummaryModel {
    languageconfig: Option<String>,
    keyboardconfig: Option<String>,
    timezoneconfig: Option<String>,
    #[tracker::no_eq]
    partitionconfig: Option<PartitionSchema>,
    userconfig: Option<UserConfig>,

    prettylanguage: Option<String>,
    prettykeyboard: Option<String>,

    #[tracker::no_eq]
    partitions: FactoryVecDeque<Partition>,

    showhostname: bool,
}

#[derive(Debug)]
pub enum SummaryMsg {
    SetConfig(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<PartitionSchema>,
        Box<Option<UserConfig>>,
    ),
    ShowHostname(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for SummaryModel {
    type Init = ();
    type Input = SummaryMsg;
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
                    gtk::Label {
                        add_css_class: "title-1",
                        #[watch]
                        set_label: &gettext("Summary"),
                        set_halign: gtk::Align::Center,
                    },
                    adw::PreferencesGroup {
                        #[watch]
                        set_title: &gettext("Language"),
                        adw::ActionRow {
                            set_activatable: false,
                            #[watch]
                            set_title: model.prettylanguage.as_ref().unwrap_or(&"None".to_string()),
                            #[watch]
                            set_subtitle: model.languageconfig.as_ref().unwrap_or(&"None".to_string()),
                        },
                    },
                    adw::PreferencesGroup {
                        #[watch]
                        set_title: &gettext("Keyboard"),
                        adw::ActionRow {
                            set_activatable: false,
                            #[watch]
                            set_title: model.prettykeyboard.as_ref().unwrap_or(&"None".to_string()),
                            #[watch]
                            set_subtitle: model.keyboardconfig.as_ref().unwrap_or(&"None".to_string()),
                        },
                    },
                    adw::PreferencesGroup {
                        #[watch]
                        set_title: &gettext("Timezone"),
                        adw::ActionRow {
                            set_activatable: false,
                            #[watch]
                            set_title: model.timezoneconfig.as_ref().unwrap_or(&"None".to_string()),
                        },
                    },

                    match &model.partitionconfig {
                        Some(PartitionSchema::FullDisk(disk)) => {
                            adw::PreferencesGroup {
                                #[watch]
                                set_title: &gettext("Partitions"),
                                adw::ActionRow {
                                    set_activatable: false,
                                    #[watch]
                                    set_title: disk,
                                    #[watch]
                                    set_subtitle: &gettext("Full Disk"),
                                    add_suffix = &gtk::Label {
                                        #[watch]
                                        set_label: &gettext("Entire disk will be formatted"),
                                    }
                                },
                            }
                        }
                        Some(PartitionSchema::Custom(_partitions)) => {
                            #[local]
                            custompartitiongroup -> adw::PreferencesGroup {
                                #[watch]
                                set_title: &gettext("Partitions")
                            }
                        }
                        None => {
                            adw::PreferencesGroup {
                                #[watch]
                                set_title: &gettext("Partitions"),
                                adw::ActionRow {
                                    set_activatable: false,
                                    set_title: "N/A",
                                },
                            }
                        }
                    },
                    adw::PreferencesGroup {
                        #[watch]
                        set_title: &gettext("User"),
                        adw::ActionRow {
                            set_activatable: false,
                            #[watch]
                            set_title: &gettext("Name"),
                            add_suffix = &gtk::Button {
                                set_halign: gtk::Align::Start,
                                set_valign: gtk::Align::Center,
                                gtk::Label {
                                    #[watch]
                                    set_markup: &if let Some(userconf) = &model.userconfig {
                                        format!("<tt>{}</tt>", userconf.name)
                                    } else {
                                        "<tt>N/A</tt>".to_string()
                                    },
                                },
                                set_can_target: false,
                            }
                        },
                        adw::ActionRow {
                            set_activatable: false,
                            #[watch]
                            set_title: &gettext("Username"),
                            add_suffix = &gtk::Button {
                                set_halign: gtk::Align::Start,
                                set_valign: gtk::Align::Center,
                                gtk::Label {
                                    #[watch]
                                    set_markup: &if let Some(userconf) = &model.userconfig {
                                        format!("<tt>{}</tt>", userconf.username)
                                    } else {
                                        "<tt>N/A</tt>".to_string()
                                    },
                                },
                                set_can_target: false,
                            }
                        },
                        adw::ActionRow {
                            set_activatable: false,
                            #[watch]
                            set_title: &gettext("Hostname"),
                            #[watch]
                            set_visible: model.showhostname,
                            add_suffix = &gtk::Button {
                                set_halign: gtk::Align::Start,
                                set_valign: gtk::Align::Center,
                                gtk::Label {
                                    #[watch]
                                    set_markup: &if let Some(userconf) = &model.userconfig {
                                        format!("<tt>{}</tt>", userconf.hostname)
                                    } else {
                                        "<tt>N/A</tt>".to_string()
                                    },
                                },
                                set_can_target: false,
                            }
                        }
                    },
                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SummaryModel {
            languageconfig: None,
            keyboardconfig: None,
            timezoneconfig: None,
            partitionconfig: None,
            userconfig: None,
            prettylanguage: None,
            prettykeyboard: None,
            partitions: FactoryVecDeque::builder()
                .launch_default()
                .detach(),
            showhostname: false,
            tracker: 0,
        };

        let custompartitiongroup = model.partitions.widget().clone();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            SummaryMsg::SetConfig(
                languageconfig,
                keyboardconfig,
                timezoneconfig,
                partitionconfig,
                userconfig,
            ) => {
                let debuguser = userconfig.clone().map(|mut user| {
                    user.password = "*****".to_string();
                    user.rootpassword = user.rootpassword.map(|_| "*****".to_string());
                    user
                });
                debug!(
                    "SetConfig: {:?}, {:?}, {:?}, {:?}, {:?}",
                    languageconfig, keyboardconfig, timezoneconfig, partitionconfig, debuguser
                );
                self.languageconfig = languageconfig;
                self.keyboardconfig = keyboardconfig;
                self.timezoneconfig = timezoneconfig;
                self.partitionconfig = partitionconfig;
                self.userconfig = *userconfig;

                if let Some(lang) = self.languageconfig.as_ref() {
                    let lang =
                        gnome_desktop::language_from_locale(lang, self.languageconfig.as_deref());
                    self.prettylanguage = lang.map(|l| l.to_string());
                }

                if let Some(keyboard) = self.keyboardconfig.as_ref() {
                    let xkb = XkbInfo::new();
                    let layout = xkb
                        .layout_info(keyboard)
                        .and_then(|x| x.0)
                        .map(|x| x.to_string());
                    self.prettykeyboard = layout;
                }

                if let Some(PartitionSchema::Custom(partitions)) = &self.partitionconfig {
                    let mut partitions_guard = self.partitions.guard();
                    partitions_guard.clear();
                    for (name, partition) in partitions {
                        partitions_guard.push_back((name.to_string(), partition.clone()));
                    }
                    partitions_guard.drop();
                }
            }
            SummaryMsg::ShowHostname(showhostname) => {
                self.showhostname = showhostname;
            }
        }
    }
}

pub struct Partition {
    name: String,
    mountpoint: Option<String>,
    format: Option<String>,
}

#[relm4::factory(pub)]
impl FactoryComponent for Partition {
    type Init = (String, CustomPartition);
    type Input = ();
    type Output = ();
    type ParentWidget = adw::PreferencesGroup;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            set_title: &self.name,
            add_suffix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_margin_start: 10,
                    gtk::Label {
                        #[watch]
                        // Translators: Leave the ':' at the end of the string, it will read Mountpoint: {disk mount location}
                        set_label: &gettext("Mountpoint:"),
                    },
                    gtk::Button {
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Center,
                        gtk::Label {
                            #[watch]
                            set_markup: &format!("<tt>{}</tt>", self.mountpoint.clone().unwrap_or_else(|| gettext("Do not mount"))),
                        },
                        set_can_target: false,
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_margin_start: 10,
                    gtk::Label {
                        #[watch]
                        // Translators: Leave the ':' at the end of the string, it will read Format: {disk format}
                        set_label: &gettext("Format:"),
                    },
                    gtk::Button {
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Center,
                        gtk::Label {
                            #[watch]
                            set_markup: &format!("<tt>{}</tt>", self.format.clone().unwrap_or_else(|| gettext("Do not format"))),
                        },
                        set_can_target: false,
                    }
                },
            }
        }
    }

    fn init_model(
        (name, partition): Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Partition {
            name,
            mountpoint: partition.mountpoint,
            format: partition.format,
        }
    }
}
