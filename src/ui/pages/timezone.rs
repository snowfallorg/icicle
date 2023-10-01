use crate::ui::window::AppMsg;
use adw::prelude::*;
use gettextrs::gettext;
use glib::TimeZone;
use gnome_desktop::{self, WallClockExt};
use log::{trace, debug};
use relm4::*;
use std::{collections::HashMap, process::Command};

#[tracker::track]
#[derive(Debug)]
pub struct TimeZoneModel {
    timezones: Vec<(String, Vec<(String, TimeZone)>)>,
    language: Option<String>,
    country: Option<String>,
    showall: bool,
    selectiongroup: gtk::CheckButton,
    selected: Option<String>,
    expanders: Vec<adw::ExpanderRow>,
    time: String,
    timelist: HashMap<TimeZone, gtk::Label>,
}

#[derive(Debug)]
pub enum TimeZoneMsg {
    ToggleShowall,
    SetSelected(Option<String>),
    SetTime(String),
    CheckSelected,
}

#[relm4::component(pub)]
impl SimpleComponent for TimeZoneModel {
    type Input = TimeZoneMsg;
    type Output = AppMsg;
    type Init = ();

    view! {
        gtk::ScrolledWindow {
            set_hexpand: true,
            set_vexpand: true,
            adw::Clamp {
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,
                    set_margin_all: 20,
                    gtk::Label {
                        #[watch]
                        set_label: &gettext("Timezone"),
                        add_css_class: "title-1"
                    },
                    #[name(tzstack)]
                    if model.showall {
                        #[local_ref]
                        tzbox -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_selection_mode: gtk::SelectionMode::None,
                        }
                    } else {
                        #[local_ref]
                        shorttzbox -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_selection_mode: gtk::SelectionMode::None,
                            connect_row_activated => move |_, row| {
                                let checkbutton = row.child().unwrap().downcast::<gtk::Box>().unwrap().last_child().unwrap().downcast::<gtk::CheckButton>().unwrap();
                                checkbutton.set_active(true);
                            },
                        }
                    },
                    gtk::Button {
                        add_css_class: "pill",
                        set_halign: gtk::Align::Center,
                        #[watch]
                        set_label: &if model.showall { gettext("Show less") } else { gettext("Show all") },
                        connect_clicked[sender] => move |_| {
                            sender.input(TimeZoneMsg::ToggleShowall);
                            sender.input(TimeZoneMsg::SetSelected(None));
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
        let mut locvec: Vec<libgweather::Location> = vec![];
        let x = libgweather::Location::world().unwrap();
        loop {
            let loc = x.next_child(locvec.last().cloned());
            if loc.is_none() {
                break;
            }
            let loc = loc.unwrap();
            locvec.push(loc);
        }

        let mut timezones: HashMap<String, Vec<(String, TimeZone)>> = HashMap::new();

        let countries = libgweather::Location::world().unwrap();

        let y = countries.timezones();

        let shorttzvec_entries = &[
            "America/New_York",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Asia/Tokyo",
            "Australia/Sydney",
            "Asia/Shanghai",
        ];

        let mut shorttzvec = vec![];
        let mut selected = gnome_desktop::WallClock::new()
            .timezone()
            .map(|x| x.identifier().to_string());
        if ["UTC", "CET", "Etc/GMT+12"].contains(&selected.as_ref().unwrap().as_str()) {
            selected = Some("America/New_York".to_string());
        }
        debug!("Selected timezone: {:?}", selected);
        
        for tz in y.get(2..).unwrap() {
            if let (Some(country), Some(region)) = (
                tz.identifier().split('/').next(),
                tz.identifier().split('/').nth(1),
            ) {
                if !timezones.contains_key(country) {
                    timezones.insert(country.to_string(), vec![]);
                }
                timezones
                    .get_mut(country)
                    .unwrap()
                    .push((region.to_string(), tz.clone()));
                if shorttzvec_entries.contains(&tz.identifier().as_str()) {
                    shorttzvec.push((format!("{}/{}", country, region), tz.clone()));
                } else if selected.as_ref() == Some(&tz.identifier().to_string()) {
                    shorttzvec.insert(0, (format!("{}/{}", country, region), tz.clone()));
                }
            }
        }

        shorttzvec.sort_by(|a, b| a.0.cmp(&b.0));

        timezones.iter_mut().for_each(|(_, vec)| {
            vec.sort();
        });

        let mut tzvec = timezones.into_iter().collect::<Vec<_>>();
        tzvec.sort_by(|a, b| a.0.cmp(&b.0));

        let mut model = TimeZoneModel {
            language: Some("en".to_string()),
            country: Some("us".to_string()),
            timezones: tzvec,
            showall: false,
            selected,
            selectiongroup: gtk::CheckButton::new(),
            expanders: vec![],
            time: String::default(),
            timelist: HashMap::new(),
            tracker: 0,
        };

        let asyncsender = sender.clone();
        relm4::spawn(async move {
            loop {
                let time = glib::DateTime::now(&glib::TimeZone::utc())
                    .unwrap()
                    .format("%H:%M")
                    .unwrap()
                    .to_string();
                asyncsender.input(TimeZoneMsg::SetTime(time));
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        let tzbox = gtk::ListBox::new();
        let shorttzbox = gtk::ListBox::new();

        let widgets = view_output!();

        for (country, zones) in &model.timezones {
            view! {
                expander = adw::ExpanderRow {
                    set_title: country,
                }
            }
            for (zone, tz) in zones {
                let timestr = if let Ok(time) = glib::DateTime::now(tz) {
                    time.format("%H:%M")
                        .unwrap_or_else(|_| glib::GString::from("??"))
                        .to_string()
                } else {
                    "??".to_string()
                };
                view! {
                    row = adw::PreferencesRow {
                        set_title: &zone.replace('_', " "),
                        // set_subtitle: &layout,
                        set_activatable: true,
                        // set_subtitle: &locale
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 6,
                            set_margin_start: 15,
                            set_margin_end: 7,
                            set_margin_top: 15,
                            set_margin_bottom: 15,
                            gtk::Box {
                                gtk::Label {
                                    set_label: &zone.replace('_', " "),
                                },
                                gtk::Separator {
                                    set_hexpand: true,
                                    set_opacity: 0.0,
                                },
                                #[name(timelabel)]
                                gtk::Label {
                                    set_label: &timestr,
                                },
                            },
                            gtk::CheckButton {
                                set_halign: gtk::Align::End,
                                set_group: Some(&model.selectiongroup),
                                connect_toggled[sender, country, zone] => move |x| {
                                    if x.is_active() {
                                        sender.input(TimeZoneMsg::SetSelected(Some(format!("{}/{}", country, zone))))
                                    }
                                }
                            }
                        }
                    }
                }
                expander
                    .first_child()
                    .unwrap()
                    .last_child()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .downcast::<gtk::ListBox>()
                    .unwrap()
                    .connect_row_activated(move |_, x| {
                        let checkbutton = x
                            .child()
                            .unwrap()
                            .downcast::<gtk::Box>()
                            .unwrap()
                            .last_child()
                            .unwrap()
                            .downcast::<gtk::CheckButton>()
                            .unwrap();
                        checkbutton.set_active(true);
                    });
                expander.add_row(&row);
                model.timelist.insert(tz.clone(), timelabel);
            }
            tzbox.append(&expander);
            model.expanders.push(expander);
        }

        for (zone, tz) in shorttzvec.iter().take(8) {
            let timestr = if let Ok(time) = glib::DateTime::now(tz) {
                time.format("%H:%M")
                    .unwrap_or_else(|_| glib::GString::from("??"))
                    .to_string()
            } else {
                "??".to_string()
            };
            view! {
                row = adw::PreferencesRow {
                    set_title: zone,
                    set_activatable: true,
                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,
                        set_margin_start: 15,
                        set_margin_end: 7,
                        set_margin_top: 15,
                        set_margin_bottom: 15,
                        gtk::Box {
                            gtk::Label {
                                set_label: zone,
                            },
                            gtk::Separator {
                                set_hexpand: true,
                                set_opacity: 0.0,
                            },
                            #[name(timelabel)]
                            gtk::Label {
                                set_label: &timestr,
                            },
                        },
                        #[name(rowbtn)]
                        gtk::CheckButton {
                            set_halign: gtk::Align::End,
                            set_group: Some(&model.selectiongroup),
                            connect_toggled[sender, zone = zone.to_string()] => move |x| {
                                if x.is_active() {
                                    sender.input(TimeZoneMsg::SetSelected(Some(zone.to_string())))
                                }
                            }
                        }
                    }
                }
            }
            shorttzbox.append(&row);
            rowbtn.set_active(Some(&zone.to_string()) == model.selected.as_ref());
            model.timelist.insert(tz.clone(), timelabel);
        }

        widgets.tzstack.set_vhomogeneous(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            TimeZoneMsg::SetSelected(layout) => {
                if layout.is_none() {
                    self.selectiongroup.set_active(true);
                    let _ = sender.output(AppMsg::SetCanGoForward(false));
                } else {
                    let _ = sender.output(AppMsg::SetCanGoForward(true));
                    let _ = sender.output(AppMsg::SetTimezoneConfig(layout.clone()));
                }
                self.selected = layout;
                if let Some(selected) = &self.selected {
                    let _ = Command::new("timedatectl")
                        .arg("--no-ask-password")
                        .arg("set-timezone")
                        .arg(selected)
                        .spawn();
                }
            }
            TimeZoneMsg::CheckSelected => {
                trace!("TimeZoneMsg::CheckSelected {}", self.selected.is_some());
                let _ = sender.output(AppMsg::SetCanGoForward(self.selected.is_some()));
            }
            TimeZoneMsg::ToggleShowall => {
                if !self.showall {
                    for expander in &self.expanders {
                        expander.set_expanded(false);
                    }
                }
                self.showall = !self.showall;
            }
            TimeZoneMsg::SetTime(time) => {
                if time != self.time {
                    self.set_time(time);
                    self.timelist.clone().iter_mut().for_each(|(tz, label)| {
                        let timestr = if let Ok(time) = glib::DateTime::now(tz) {
                            time.format("%H:%M")
                                .unwrap_or_else(|_| glib::GString::from("??"))
                                .to_string()
                        } else {
                            "??".to_string()
                        };
                        label.set_label(&timestr);
                    });
                }
            }
        }
    }
}
