use crate::ui::window::AppMsg;
use adw::prelude::*;
use relm4::{actions::*, factory::*, *};
use std::collections::HashMap;

pub struct BaseModel {}

#[derive(Debug)]
pub enum BaseMsg {}

#[relm4::component(pub)]
impl SimpleComponent for BaseModel {
    type Init = ();
    type Input = BaseMsg;
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
                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = BaseModel {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {}
    }
}
