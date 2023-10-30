use super::window::AppMsg;
use adw::prelude::*;
use gettextrs::gettext;
use relm4::*;

#[tracker::track]
pub struct QuitDialogModel {
    hidden: bool,
}

#[derive(Debug)]
pub enum QuitDialogMsg {
    Show,
    Cancel,
    Quit,
}

#[relm4::component(pub)]
impl SimpleComponent for QuitDialogModel {
    type Init = gtk::Window;
    type Input = QuitDialogMsg;
    type Output = AppMsg;
    type Widgets = QuitCheckWidgets;

    view! {
        dialog = adw::MessageDialog {
            set_transient_for: Some(&init),
            set_modal: true,
            #[track(model.changed(QuitDialogModel::hidden()))]
            set_visible: !model.hidden,
            set_resizable: false,
            #[watch]
            set_heading: Some(&gettext("Quit Installation")),
            #[watch]
            set_body: &gettext("Quitting while the installation is in progress may leave your system in an unbootable state!"),
            set_default_width: 500,
            add_response: ("cancel", &gettext("Cancel")),
            add_response: ("quit", &gettext("Quit")),
            #[watch]
            set_response_label: ("cancel", &gettext("Cancel")),
            #[watch]
            set_response_label: ("quit", &gettext("Quit")),
            set_response_appearance: ("quit", adw::ResponseAppearance::Destructive),
            connect_close_request => move |_| {
                glib::Propagation::Stop
            },
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = QuitDialogModel {
            hidden: true,
            tracker: 0,
        };
        let widgets = view_output!();

        widgets.dialog.connect_response(None, move |_, resp| {
            sender.input(match resp {
                "cancel" => QuitDialogMsg::Cancel,
                "quit" => QuitDialogMsg::Quit,
                _ => unreachable!(),
            })
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            QuitDialogMsg::Show => {
                self.set_hidden(false);
            }
            QuitDialogMsg::Cancel => {
                self.set_hidden(true);
            }
            QuitDialogMsg::Quit => {
                relm4::main_application().quit();
            }
        }
    }
}
