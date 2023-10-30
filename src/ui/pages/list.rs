use crate::{ui::window::AppMsg, utils::parse::Choice};
use adw::prelude::*;
use gettextrs::gettext;
use relm4::{factory::*, *};
use std::collections::HashMap;

#[tracker::track]
pub struct ListModel {
    id: String,
    title: String,
    #[tracker::no_eq]
    list: FactoryVecDeque<ListItem>,
    #[tracker::no_eq]
    choices: Vec<(String, Choice)>,
    selected: Vec<String>,
    group: Option<gtk::CheckButton>,
    required: bool,
    locale: Option<String>,
}

#[derive(Debug)]
pub enum ListMsg {
    CheckSelected,
    Select(String),
    Deselect(String),
    SetLocale(Option<String>),
}

pub struct ListInit {
    pub id: String,
    pub multiple: bool,
    pub required: bool,
    pub title: String,
    pub choices: Vec<HashMap<String, Choice>>,
}

#[relm4::component(pub)]
impl SimpleComponent for ListModel {
    type Init = ListInit;
    type Input = ListMsg;
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
                        #[track(model.changed(ListModel::locale()))]
                        set_label: &gettext(&model.title),
                    },
                    #[local_ref]
                    group -> adw::PreferencesGroup {

                    },
                    gtk::Button {
                        add_css_class: "pill",
                        set_halign: gtk::Align::Center,
                        #[track(model.changed(ListModel::locale()))]
                        set_label: &gettext("Clear"),
                        #[watch]
                        set_visible: !model.required && model.group.is_some(),
                        connect_clicked[group = model.group.clone()] => move |_| {
                            if let Some(group) = &group {
                                group.activate();
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut choices = vec![];

        for choice in init.choices {
            for (key, value) in choice {
                choices.push((key, value));
            }
        }

        let mut model = ListModel {
            id: init.id,
            title: init.title,
            list: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    ListItemMsg::Select(key) => ListMsg::Select(key),
                    ListItemMsg::Deselect(key) => ListMsg::Deselect(key),
                }),
            choices,
            selected: Vec::new(),
            required: init.required,
            group: if init.multiple {
                None
            } else {
                Some(gtk::CheckButton::new())
            },
            locale: None,
            tracker: 0,
        };

        let mut list_guard = model.list.guard();
        for (key, choice) in &model.choices {
            let item = ListItem {
                title: key.to_string(),
                description: choice.description.clone().unwrap_or_default(),
                group: model.group.clone(),
                locale: model.locale.clone(),
                tracker: 0,
            };
            list_guard.push_back(item);
        }
        list_guard.drop();
        let group = model.list.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            ListMsg::CheckSelected => {
                let cangoforward = if self.required {
                    !self.selected.is_empty()
                } else {
                    true
                };
                let _ = sender.output(AppMsg::SetCanGoForward(cangoforward));
            }
            ListMsg::Select(key) => {
                self.selected.push(key);
                sender.input(ListMsg::CheckSelected);
                let mut selected = self.choices.iter().cloned().collect::<HashMap<_, _>>();
                selected.retain(|k, _| self.selected.contains(k));
                let _ = sender.output(AppMsg::SetListConfig(self.id.to_string(), selected));
            }
            ListMsg::Deselect(key) => {
                self.selected.retain(|k| k != &key);
                sender.input(ListMsg::CheckSelected);
                let mut selected = self.choices.iter().cloned().collect::<HashMap<_, _>>();
                selected.retain(|k, _| self.selected.contains(k));
                let _ = sender.output(AppMsg::SetListConfig(self.id.to_string(), selected));
            }
            ListMsg::SetLocale(locale) => {
                self.set_locale(locale);
                let mut list_guard = self.list.guard();
                for item in list_guard.iter_mut() {
                    item.set_locale(self.locale.clone());
                }
                list_guard.drop();
            }
        }
    }
}

#[tracker::track]
pub struct ListItem {
    title: String,
    description: String,
    group: Option<gtk::CheckButton>,
    locale: Option<String>,
}

#[derive(Debug)]
pub enum ListItemMsg {
    Select(String),
    Deselect(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for ListItem {
    type Init = ListItem;
    type Input = ();
    type Output = ListItemMsg;
    type ParentWidget = adw::PreferencesGroup;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            #[track(self.changed(ListItem::locale()))]
            set_title: &gettext(&self.title),
            #[track(self.changed(ListItem::locale()))]
            set_subtitle: &gettext(&self.description),
            set_activatable: true,
            connect_activated[checkbtn] => move |_| {
                checkbtn.activate();
            },
            #[name(checkbtn)]
            add_suffix = &gtk::CheckButton {
                set_group: self.group.as_ref(),
                connect_toggled[sender, title = self.title.to_string()] => move |checkbtn| {
                    if checkbtn.is_active() {
                        let _ = sender.output(ListItemMsg::Select(title.to_string()));
                    } else {
                        let _ = sender.output(ListItemMsg::Deselect(title.to_string()));
                    }
                },
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }

    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}
