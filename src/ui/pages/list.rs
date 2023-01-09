use crate::{ui::window::AppMsg, utils::parse::Choice};
use adw::prelude::*;
use gettextrs::gettext;
use relm4::{factory::*, *};
use std::collections::HashMap;

pub struct ListModel {
    id: String,
    title: String,
    list: FactoryVecDeque<ListItem>,
    choices: Vec<(String, Choice)>,
    selected: Vec<String>,
    group: Option<gtk::CheckButton>,
    required: bool,
}

#[derive(Debug)]
pub enum ListMsg {
    CheckSelected,
    Select(String),
    Deselect(String),
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
                        set_label: &model.title,
                    },
                    #[local_ref]
                    group -> adw::PreferencesGroup {

                    },
                    gtk::Button {
                        add_css_class: "pill",
                        set_halign: gtk::Align::Center,
                        #[watch]
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
            list: FactoryVecDeque::new(adw::PreferencesGroup::new(), sender.input_sender()),
            choices,
            selected: Vec::new(),
            required: init.required,
            group: if init.multiple {
                None
            } else {
                Some(gtk::CheckButton::new())
            },
        };

        let mut list_guard = model.list.guard();
        for (key, choice) in &model.choices {
            let item = ListItem {
                title: key.to_string(),
                description: choice.description.clone().unwrap_or_default(),
                group: model.group.clone(),
            };
            list_guard.push_back(item);
        }
        list_guard.drop();
        let group = model.list.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
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
        }
    }
}

pub struct ListItem {
    title: String,
    description: String,
    group: Option<gtk::CheckButton>,
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
    type ParentInput = ListMsg;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            set_title: &self.title,
            set_subtitle: &self.description,
            set_activatable: true,
            connect_activated[checkbtn] => move |_| {
                checkbtn.activate();
            },
            #[name(checkbtn)]
            add_suffix = &gtk::CheckButton {
                set_group: self.group.as_ref(),
                connect_toggled[sender, title = self.title.to_string()] => move |checkbtn| {
                    if checkbtn.is_active() {
                        sender.output(ListItemMsg::Select(title.to_string()));
                    } else {
                        sender.output(ListItemMsg::Deselect(title.to_string()));
                    }
                },
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }

    fn output_to_parent_input(output: Self::Output) -> Option<Self::ParentInput> {
        Some(match output {
            ListItemMsg::Select(key) => ListMsg::Select(key),
            ListItemMsg::Deselect(key) => ListMsg::Deselect(key),
        })
    }
}
