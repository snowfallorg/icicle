use crate::{config::SYSCONFDIR, ui::window::AppMsg, utils::parse::parse_branding};
use adw::prelude::*;
use gettextrs::gettext;
use gtk::gio;
use log::{debug, error};
use relm4::{factory::*, *};
use std::fs::File;
use vte::{self, TerminalExt, TerminalExtManual};

pub struct InstallModel {
    terminal: vte::Terminal,
    progressbar: gtk::ProgressBar,
    showterminal: bool,
    installing: bool,
    slides: FactoryVecDeque<InstallSlide>,
    locale: Option<String>,
}

#[derive(Debug)]
pub enum InstallMsg {
    Pulse,
    NextSlide,
    ToggleTerminal,
    Echo(String),
    Install(Vec<String>),
    VTEOutput(i32),
    SetLocale(Option<String>),
    PostInstall(Vec<String>),
}

pub static INSTALL_BROKER: MessageBroker<InstallMsg> = MessageBroker::new();

#[relm4::component(pub)]
impl SimpleComponent for InstallModel {
    type Init = String;
    type Input = InstallMsg;
    type Output = AppMsg;

    view! {
        gtk::ScrolledWindow {
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 20,
                set_margin_all: 20,
                if model.showterminal {
                    gtk::Box {
                        gtk::Frame {
                            #[local_ref]
                            terminal -> vte::Terminal {
                                set_hexpand: true,
                                connect_child_exited[sender] => move |_term, status| {
                                    sender.input(InstallMsg::VTEOutput(status));
                                },
                                set_input_enabled: false,
                            }
                        }
                    }
                } else {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                        #[local_ref]
                        carousel -> adw::Carousel {
                            set_vexpand: true,
                            set_halign: gtk::Align::Fill,
                            set_valign: gtk::Align::Fill,
                        },
                        adw::CarouselIndicatorDots {
                            set_halign: gtk::Align::Center,
                            set_hexpand: true,
                            set_carousel: Some(carousel)
                        }
                    }

                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,
                    #[local_ref]
                    progressbar -> gtk::ProgressBar {
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Fill,
                        set_pulse_step: 0.02,
                    },
                    gtk::Button {
                        set_valign: gtk::Align::Center,
                        add_css_class: "circular",
                        set_icon_name: "utilities-terminal-symbolic",
                        connect_clicked[sender] => move |_| {
                            sender.input(InstallMsg::ToggleTerminal);
                        }
                    }
                }
            }
        }
    }

    fn init(
        branding: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = InstallModel {
            terminal: vte::Terminal::new(),
            showterminal: false,
            progressbar: gtk::ProgressBar::new(),
            installing: false,
            slides: FactoryVecDeque::builder()
                .launch_default()
                .detach(),
            locale: None,
        };

        if let Ok(brandingconfig) = parse_branding(&branding) {
            let mut slides_guard = model.slides.guard();
            for slide in brandingconfig.slides {
                slides_guard.push_back(InstallSlide {
                    title: slide.title,
                    subtitle: slide.subtitle,
                    image: format!(
                        "{}/icicle/branding/{}/{}",
                        SYSCONFDIR, branding, slide.image
                    ),
                    locale: model.locale.clone(),
                    tracker: 0,
                });
            }
            slides_guard.drop();
        }

        let terminal = &model.terminal;
        let progressbar = &model.progressbar;
        let carousel = model.slides.widget();
        let widgets = view_output!();
        let pulsesender = sender.clone();
        relm4::spawn(async move {
            loop {
                pulsesender.input(InstallMsg::Pulse);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
        relm4::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(12)).await;
                sender.input(InstallMsg::NextSlide);
            }
        });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            InstallMsg::Pulse => {
                self.progressbar.pulse();
            }
            InstallMsg::NextSlide => {
                let npages = self.slides.widget().n_pages();
                let currentpage = self.slides.widget().position();
                if currentpage.fract() == 0.0 {
                    let next = (self.slides.widget().position() as u32 + 1) % npages;
                    let next = self.slides.widget().nth_page(next);
                    self.slides.widget().scroll_to(&next, true);
                }
            }
            InstallMsg::ToggleTerminal => {
                self.showterminal = !self.showterminal;
            }
            InstallMsg::Echo(s) => {
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &["/usr/bin/env", "echo", &s],
                    &[],
                    adw::glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |_| (),
                );
            }
            InstallMsg::Install(cmds) => {
                debug!("Installing: {:?}", cmds);
                self.installing = true;
                let cmds: Vec<&str> = cmds.iter().map(|x| &**x).collect();
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &cmds,
                    &[],
                    adw::glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |err| (debug!("VTE Install: {:?}", err)),
                );
            }
            InstallMsg::PostInstall(cmds) => {
                debug!("PostInstall command: {:?}", cmds);
                self.installing = false;
                let cmds: Vec<&str> = cmds.iter().map(|x| &**x).collect();
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &cmds,
                    &[],
                    adw::glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |err| (debug!("VTE postinstall: {:?}", err)),
                );
            }
            InstallMsg::VTEOutput(status) => {
                debug!("VTE command exited with status: {}", status);
                if let Ok(file) = File::create("/tmp/icicle-term.log") {
                    let output = gio::WriteOutputStream::new(file);
                    if let Err(e) = self.terminal.write_contents_sync(
                        &output,
                        vte::WriteFlags::Default,
                        gio::Cancellable::NONE,
                    ) {
                        error!("{:?}", e);
                    }
                    let _ = output.flush(gio::Cancellable::NONE);
                }
                if self.installing {
                    if status == 0 {
                        debug!("Installation Success!");
                        let _ = sender.output(AppMsg::FinishInstall);
                    } else {
                        debug!("Installation Failed!");
                        let _ = sender.output(AppMsg::Error);
                    }
                } else {
                    if status == 0 {
                        debug!("Post install command success!");
                        let _ = sender.output(AppMsg::RunNextCommand);
                    } else {
                        debug!("Post install command failed!");
                        let _ = sender.output(AppMsg::Error);
                    }
                }
            }
            InstallMsg::SetLocale(locale) => {
                self.locale = locale;
                let mut slides_guard = self.slides.guard();
                for item in slides_guard.iter_mut() {
                    item.set_locale(self.locale.clone());
                }
                slides_guard.drop();
            }
        }
    }
}

#[derive(Debug)]
#[tracker::track]
pub struct InstallSlide {
    title: String,
    subtitle: String,
    image: String,
    locale: Option<String>,
}

#[relm4::factory(pub)]
impl FactoryComponent for InstallSlide {
    type Init = InstallSlide;
    type Input = ();
    type Output = ();
    type ParentWidget = adw::Carousel;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_hexpand: true,
            set_vexpand: true,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 15,
            gtk::Label {
                set_halign: gtk::Align::Center,
                add_css_class: "title-3",
                #[track(self.changed(InstallSlide::locale()))]
                set_label: &gettext(&self.title)
            },
            gtk::Label {
                set_halign: gtk::Align::Center,
                #[track(self.changed(InstallSlide::locale()))]
                set_label: &gettext(&self.subtitle)
            },
            gtk::Picture {
                set_margin_all: 50,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_filename: Some(&self.image)
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }

    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}
