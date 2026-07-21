use crate::{ui::window::AppMsg, utils::language::get_languages};
use adw::prelude::*;
use gettextrs::gettext;
use log::{info, trace};
use relm4::*;

#[tracker::track]
pub struct WelcomeModel {
    showall: bool,
    selected: Option<String>,
    selectiongroup: gtk::CheckButton,
    expanders: Vec<adw::ExpanderRow>,
}

#[derive(Debug)]
pub enum WelcomeMsg {
    ToggleShowall,
    SetSelected(Option<String>),
    CheckSelected,
}

#[relm4::component(pub)]
impl SimpleComponent for WelcomeModel {
    type Init = ();
    type Input = WelcomeMsg;
    type Output = AppMsg;

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
                        set_label: &gettext("Welcome!"),
                        add_css_class: "title-1",
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &gettext("Choose a language"),
                        add_css_class: "title-3"
                    },
                    #[name(langstack)]
                    if model.showall {
                        #[local_ref]
                        langbox -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_selection_mode: gtk::SelectionMode::None,
                            connect_row_activated => move |_, row| {
                                let checkbutton = row.child().unwrap().downcast::<gtk::Box>().unwrap().last_child().unwrap().downcast::<gtk::CheckButton>().unwrap();
                                checkbutton.set_active(true);
                            },
                        }
                    } else {
                        #[local_ref]
                        shortlangbox -> gtk::ListBox {
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
                            sender.input(WelcomeMsg::ToggleShowall);
                            sender.input(WelcomeMsg::SetSelected(None));
                        }
                    }

                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = WelcomeModel {
            showall: false,
            selected: None,
            selectiongroup: gtk::CheckButton::new(),
            expanders: vec![],
            tracker: 0,
        };

        // List of 6 popular languages
        let shortlangs = vec![
            "en_US.UTF-8",
            "es_ES.UTF-8",
            "fr_FR.UTF-8",
            "de_DE.UTF-8",
            "it_IT.UTF-8",
            "pt_BR.UTF-8",
        ];

        let defaultlang = "en_US.UTF-8";
        model.selected = Some(defaultlang.to_string());

        let langbox = gtk::ListBox::new();
        let shortlangbox = gtk::ListBox::new();

        let mut languages = get_languages().into_iter().collect::<Vec<_>>();
        languages.sort_by(|a, b| a.0.cmp(&b.0));
        for (title, languages) in languages {
            for locale in &shortlangs {
                if let Some(title) = languages.get(&locale.to_string()) {
                    view! {
                        row = adw::PreferencesRow {
                            set_title: locale,
                            set_activatable: true,
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 6,
                                set_margin_start: 15,
                                set_margin_end: 7,
                                set_margin_top: 15,
                                set_margin_bottom: 15,
                                gtk::Label {
                                    set_label: title,
                                },
                                gtk::Separator {
                                    set_hexpand: true,
                                    set_opacity: 0.0,
                                },
                                #[name(rowbtn)]
                                gtk::CheckButton {
                                    set_halign: gtk::Align::End,
                                    set_group: Some(&model.selectiongroup),
                                    connect_toggled[sender, locale = locale.to_string()] => move |x| {
                                        if x.is_active() {
                                            sender.input(WelcomeMsg::SetSelected(Some(locale.to_string())))
                                        }
                                    }
                                }
                            }
                        }
                    };
                    shortlangbox.append(&row);
                    rowbtn.set_active(locale == &defaultlang);
                }
            }

            if languages.len() > 1 {
                view! {
                    expander = adw::ExpanderRow {
                        set_title: &title,
                    }
                };
                langbox.append(&expander);

                let mut langvec = languages.into_iter().collect::<Vec<_>>();
                langvec.sort_by(|a, b| a.1.cmp(&b.1));
                for (locale, title) in &langvec {
                    view! {
                        row = adw::PreferencesRow {
                            set_title: locale,
                            set_activatable: true,
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 6,
                                set_margin_start: 15,
                                set_margin_end: 7,
                                set_margin_top: 15,
                                set_margin_bottom: 15,
                                gtk::Label {
                                    set_label: title,
                                },
                                gtk::Separator {
                                    set_hexpand: true,
                                    set_opacity: 0.0,
                                },
                                gtk::CheckButton {
                                    set_halign: gtk::Align::End,
                                    set_group: Some(&model.selectiongroup),
                                    connect_toggled[sender, locale] => move |x| {
                                        if x.is_active() {
                                            sender.input(WelcomeMsg::SetSelected(Some(locale.to_string())))
                                        }
                                    }
                                }
                            }
                        }
                    };
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
                }
                model.expanders.push(expander);
            } else {
                let (locale, title) = languages.into_iter().next().unwrap();
                view! {
                    row = adw::PreferencesRow {
                        set_title: &locale,
                        set_activatable: true,
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 6,
                            set_margin_start: 15,
                            set_margin_end: 7,
                            set_margin_top: 15,
                            set_margin_bottom: 15,
                            gtk::Label {
                                set_label: &title,
                            },
                            gtk::Separator {
                                set_hexpand: true,
                                set_opacity: 0.0,
                            },
                            gtk::CheckButton {
                                set_halign: gtk::Align::End,
                                set_group: Some(&model.selectiongroup),
                                connect_toggled[sender, locale] => move |x| {
                                    if x.is_active() {
                                        sender.input(WelcomeMsg::SetSelected(Some(locale.to_string())))
                                    }
                                }
                            }
                        }
                    }
                };
                langbox.append(&row);
            }
        }

        let widgets = view_output!();
        widgets.langstack.set_vhomogeneous(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            WelcomeMsg::ToggleShowall => {
                if !self.showall {
                    for expander in &self.expanders {
                        expander.set_expanded(false);
                    }
                }
                self.set_showall(!self.showall);
            }
            WelcomeMsg::SetSelected(x) => {
                info!("Selected language: {:?}", x);
                if let Some(lang) = &x {
                    let _ = sender.output(AppMsg::SetCanGoForward(true));
                    let _ = sender.output(AppMsg::SetLanguageConfig(Some(lang.to_string())));
                } else {
                    self.selectiongroup.set_active(true);
                    let _ = sender.output(AppMsg::SetCanGoForward(false));
                }
                self.selected = x;
                gettextrs::setlocale(
                    gettextrs::LocaleCategory::LcAll,
                    self.selected
                        .as_deref()
                        .unwrap_or_default()
                        .split('.')
                        .next()
                        .unwrap_or_default(),
                );
            }
            WelcomeMsg::CheckSelected => {
                trace!("WelcomeMsg::CheckSelected {}", self.selected.is_some());
                let _ = sender.output(AppMsg::SetCanGoForward(self.selected.is_some()));
            }
        }
    }
}
