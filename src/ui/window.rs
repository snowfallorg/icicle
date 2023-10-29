use super::pages::{
    error::ErrorModel,
    install::{InstallModel, InstallMsg},
    keyboard::{KeyboardModel, KeyboardMsg},
    list::ListModel,
    partitions::{PartitionMsg, PartitionSchema},
    summary::{SummaryModel, SummaryMsg},
    timezone::TimeZoneMsg,
    user::UserModel,
    welcome::WelcomeMsg,
};
use crate::{
    ui::{
        pages::{
            error::ErrorMsg,
            install::INSTALL_BROKER,
            list::{ListInit, ListMsg},
            partitions::{PartitionModel, PARTITION_BROKER},
            timezone::TimeZoneModel,
            user::UserMsg,
            welcome::WelcomeModel,
        },
        quitdialog::{QuitDialogModel, QuitDialogMsg},
    },
    utils::{
        i18n::i18n_f,
        install::{InstallAsyncModel, InstallAsyncMsg},
        language::{get_country, get_lang},
        parse::{parse_config, Choice, ChoiceEnum, IcicleConfig, InstallationConfig, StepType},
    },
};
use adw::prelude::*;
use gettextrs::gettext;
use log::{debug, error, info, trace, warn};
use relm4::*;
use std::{collections::HashMap, convert::identity, process::Command};

#[tracker::track]
pub struct AppModel {
    page: StackPage,
    #[tracker::no_eq]
    config: IcicleConfig,
    #[tracker::no_eq]
    installconfig: Option<InstallationConfig>,
    #[tracker::no_eq]
    welcome: Controller<WelcomeModel>,
    #[tracker::no_eq]
    keyboard: Controller<KeyboardModel>,
    #[tracker::no_eq]
    timezone: Controller<TimeZoneModel>,
    #[tracker::no_eq]
    partition: Controller<PartitionModel>,
    #[tracker::no_eq]
    user: Controller<UserModel>,
    #[tracker::no_eq]
    summary: Controller<SummaryModel>,
    #[tracker::no_eq]
    install: Controller<InstallModel>,
    #[tracker::no_eq]
    list: HashMap<String, Controller<ListModel>>,
    #[tracker::no_eq]
    listconfig: HashMap<String, HashMap<String, Choice>>,
    #[tracker::no_eq]
    error: Controller<ErrorModel>,
    #[tracker::no_eq]
    quitdialog: Controller<QuitDialogModel>,

    can_go_back: bool,
    can_go_forward: bool,
    carousel: adw::Carousel,
    #[tracker::no_eq]
    carouselpages: HashMap<usize, StepType>,
    current_page: u32,

    languageconfig: Option<String>,
    keyboardconfig: Option<String>,
    timezoneconfig: Option<String>,
    #[tracker::no_eq]
    partitionconfig: Option<PartitionSchema>,
    userconfig: Option<UserConfig>,

    #[tracker::no_eq]
    installworker: WorkerController<InstallAsyncModel>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserConfig {
    pub name: String,
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub rootpassword: Option<String>,
    pub autologin: bool,
}

#[derive(Debug)]
pub enum AppMsg {
    ChangePage(u32),
    SetCanGoBack(bool),
    SetCanGoForward(bool),
    SetStackPage(StackPage),
    SetStackPageConfig(StackPage, Option<InstallationConfig>),
    SetLanguageConfig(Option<String>),
    SetKeyboardConfig(Option<String>),
    SetTimezoneConfig(Option<String>),
    SetPartitionConfig(Option<PartitionSchema>),
    SetUserConfig(Option<UserConfig>),

    SetListConfig(String, HashMap<String, Choice>),

    Install,
    FinishInstall,
    RunNextCommand,

    Finished,
    Error,
}

#[derive(Debug)]
pub enum AppAsyncMsg {
    SetPage(StackPage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackPage {
    FrontPage,
    Carousel,
    Install,
    Finished,
    Error,
    NoInternet,
}

#[relm4::component(pub)]
#[allow(unused_parens)] // For relm4 match stack macro
impl Component for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = AppAsyncMsg;

    view! {
        #[name(main_window)]
        adw::ApplicationWindow {
            set_default_width: 900,
            set_default_height: 800,
            connect_close_request[quitdialog = model.quitdialog.sender().clone()] => move |_| {
                debug!("Caught close request: {}", model.page == StackPage::Install);
                if model.page == StackPage::Install {
                    let _ = quitdialog.send(QuitDialogMsg::Show);
                    glib::Propagation::Stop
                } else {            
                    relm4::main_application().quit();
                    glib::Propagation::Proceed
                }
            },
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,
                set_orientation: gtk::Orientation::Vertical,
                adw::HeaderBar {
                    add_css_class: "flat",
                    #[wrap(Some)]
                    #[transition(Crossfade)]
                    set_title_widget = match model.page {
                        (StackPage::FrontPage | StackPage::NoInternet) => {
                            gtk::Label {
                                #[watch]
                                // Translators: Do NOT translate the '{}'
                                // The string reads "{distribution name} Installer"
                                set_label: &i18n_f("{} Installer", &[&model.config.distribution_name])
                            }
                        },
                        StackPage::Carousel => {
                            adw::CarouselIndicatorDots {
                                set_halign: gtk::Align::Center,
                                set_hexpand: true,
                                set_carousel: Some(main_carousel)
                            }
                        },
                        StackPage::Install => {
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Installing…")
                            }
                        },
                        StackPage::Finished => {
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Installation Finished")
                            }
                        },
                        StackPage::Error => {
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Installation Failed")
                            }
                        }
                    }
                },

                #[transition(SlideLeftRight)]
                match model.page {
                    StackPage::FrontPage => {
                        gtk::Box {
                            set_margin_all: 20,
                            #[local_ref]
                            selectbox -> gtk::FlowBox {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_hexpand: true,
                                set_column_spacing: 20,
                                set_row_spacing: 20,
                                set_selection_mode: gtk::SelectionMode::None,
                                #[watch]
                                set_max_children_per_line: selectbox.iter_children().count() as u32,
                                set_homogeneous: true,
                            }
                        }
                    },
                    StackPage::Carousel => {
                        gtk::Overlay {
                            #[local_ref]
                            main_carousel -> adw::Carousel {
                                set_interactive: false,
                                set_halign: gtk::Align::Fill,
                                set_valign: gtk::Align::Fill,
                                set_hexpand: true,
                                set_vexpand: true,
                            },
                            add_overlay = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::Crossfade,
                                #[watch]
                                set_reveal_child: model.can_go_back && model.current_page > 0,
                                set_halign: gtk::Align::Start,
                                set_valign: gtk::Align::Center,
                                set_margin_all: 20,
                                gtk::Button {
                                    set_can_focus: false,
                                    set_can_target: true,
                                    set_height_request: 40,
                                    set_width_request: 40,
                                    add_css_class: "circular",
                                    set_halign: gtk::Align::Start,
                                    set_valign: gtk::Align::Center,
                                    set_icon_name: "go-previous-symbolic",
                                    connect_clicked[main_carousel, sender] => move |_| {
                                        let i = adw::Carousel::position(&main_carousel) as u32;
                                        if i > 0 {
                                            let w = main_carousel.nth_page(i-1);
                                            main_carousel.scroll_to(&w, true);
                                        }
                                        sender.input(AppMsg::ChangePage(i - 1));
                                    }
                                }
                            },
                            add_overlay = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::Crossfade,
                                #[watch]
                                set_reveal_child: model.can_go_forward,
                                set_halign: gtk::Align::End,
                                set_valign: gtk::Align::Center,
                                set_margin_all: 20,
                                gtk::Button {
                                    set_can_focus: false,
                                    set_can_target: true,
                                    set_height_request: 40,
                                    set_width_request: 40,
                                    #[watch]
                                    set_css_classes: if model.current_page == main_carousel.n_pages() - 1 { &["circular", "suggested-action"] } else { &["circular"] },
                                    set_halign: gtk::Align::Start,
                                    set_valign: gtk::Align::Center,
                                    set_icon_name: "go-next-symbolic",
                                    connect_clicked[main_carousel, sender] => move |_| {
                                        let i = adw::Carousel::position(&main_carousel) as u32;
                                        if i < main_carousel.n_pages() -1 {
                                            let w = main_carousel.nth_page(i+1);
                                            main_carousel.scroll_to(&w, true);
                                            sender.input(AppMsg::ChangePage(i + 1));
                                        } else {
                                            sender.input(AppMsg::SetStackPage(StackPage::Install));
                                            sender.input(AppMsg::Install);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    StackPage::Install => {
                        #[local]
                        installpage -> gtk::ScrolledWindow {}
                    }
                    StackPage::Finished => {
                        gtk::ScrolledWindow {
                            adw::Clamp {
                                gtk::Box {
                                    set_hexpand: true,
                                    set_vexpand: true,
                                    set_valign: gtk::Align::Center,
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 30,
                                    set_margin_all: 20,
                                    gtk::Label {
                                        add_css_class: "title-1",
                                        #[watch]
                                        set_label: &gettext("Finished!"),
                                    },
                                    gtk::Image {
                                        add_css_class: "success",
                                        set_icon_name: Some("emblem-ok-symbolic"),
                                        set_pixel_size: 256,
                                    },
                                    gtk::Button {
                                        add_css_class: "suggested-action",
                                        add_css_class: "pill",
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        #[watch]
                                        set_label: &gettext("Reboot"),
                                        connect_clicked => move |_| {
                                            let _ = Command::new("systemctl").arg("reboot").arg("-i").spawn();
                                        }
                                    }
                                }
                            }
                        }
                    },
                    StackPage::Error => {
                        #[local]
                        errorpage -> gtk::ScrolledWindow {}
                    },
                    StackPage::NoInternet => {
                        adw::StatusPage {
                            set_icon_name: Some("network-wireless-offline-symbolic"),
                            set_title: &gettext("No Internet"),
                            set_description: Some(&gettext("Please connect to the Internet to continue")),
                        }
                    }
                }
            }
        }
    }

    fn init(
        _application: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = parse_config().expect("Failed to parse config");
        let welcomepage = WelcomeModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Welcome page launched");
        let keyboardpage = KeyboardModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Keyboard page launched");
        let timezonepage = TimeZoneModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Timezone page launched");
        let partitionpage = PartitionModel::builder()
            .launch_with_broker((), &PARTITION_BROKER)
            .forward(sender.input_sender(), identity);
        println!("Partition page launched");
        let userpage = UserModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("User page launched");
        let summarypage = SummaryModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Summary page launched");
        let installpage = InstallModel::builder()
            .launch_with_broker(config.branding.to_string(), &INSTALL_BROKER)
            .forward(sender.input_sender(), identity);
        println!("Install page launched");
        let installworker = InstallAsyncModel::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);
        println!("Install worker launched");
        let errorpage = ErrorModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Error page launched");
        let quitdialog = QuitDialogModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        println!("Quit dialog launched");

        let res = reqwest::blocking::get(&config.internet_check_url);
        let startpage = if let Ok(res) = res {
            if res.status().is_success() {
                StackPage::FrontPage
            } else {
                StackPage::NoInternet
            }
        } else {
            StackPage::NoInternet
        };

        if startpage == StackPage::NoInternet {
            debug!("Waiting for internet connection…");
            let configclone = config.clone();
            sender.oneshot_command(async move {
                loop {
                    let client = reqwest::Client::new();
                    let res = client.get(&configclone.internet_check_url).send().await;
                    if let Ok(res) = res {
                        if res.status().is_success() {
                            debug!("Internet connection found!");
                            break;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                AppAsyncMsg::SetPage(StackPage::FrontPage)
            });
        }

        let model = AppModel {
            page: startpage,
            config,
            installconfig: None,
            welcome: welcomepage,
            keyboard: keyboardpage,
            timezone: timezonepage,
            partition: partitionpage,
            user: userpage,
            summary: summarypage,
            install: installpage,
            list: HashMap::new(),
            listconfig: HashMap::new(),
            error: errorpage,
            quitdialog,
            can_go_back: true,
            can_go_forward: true,
            carousel: adw::Carousel::new(),
            carouselpages: HashMap::new(),
            current_page: 0,
            languageconfig: None,
            keyboardconfig: None,
            timezoneconfig: None,
            partitionconfig: None,
            userconfig: None,
            installworker,
            tracker: 0,
        };

        let main_carousel = &model.carousel;
        let selectbox = gtk::FlowBox::new();

        for item in &model.config.choices {
            match item {
                ChoiceEnum::Configuration { file: _, config } => {
                    view! {
                        button = gtk::Button {
                            set_width_request: 200,
                            set_height_request: 200,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            connect_clicked[sender, config] => move |_| {
                                sender.input(AppMsg::SetStackPageConfig(StackPage::Carousel, Some(config.clone())));
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_spacing: 10,
                                set_margin_all: 10,
                                gtk::Image {
                                    set_icon_name: Some(&config.config_logo),
                                    set_pixel_size: 80,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                },
                                gtk::Label {
                                    set_label: &gettext(&config.config_name),
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    set_wrap: true,
                                    set_justify: gtk::Justification::Center,
                                }
                            }

                        }
                    }
                    selectbox.append(&button);
                }
                ChoiceEnum::Live => {
                    view! {
                        button = gtk::Button {
                            set_width_request: 200,
                            set_height_request: 200,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            connect_clicked => move |_| {
                                relm4::main_application().quit();
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_spacing: 10,
                                set_margin_all: 10,
                                gtk::Image {
                                    set_icon_name: Some("preferences-desktop-display-symbolic"),
                                    set_pixel_size: 80,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                },
                                gtk::Label {
                                    // Translators: Do NOT translate the '{}'
                                    // The string reads "Try {distribution name} live"
                                    set_label: i18n_f("Try {} live", &[&model.config.distribution_name]).as_str(),
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    set_wrap: true,
                                    set_justify: gtk::Justification::Center,
                                }
                            }

                        }
                    }
                    selectbox.append(&button);
                }
            }
        }

        let installpage = model.install.widget().clone();
        let errorpage = model.error.widget().clone();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.reset();
        match msg {
            AppMsg::ChangePage(page) => {
                trace!("AppMsg::ChangePage: {}", page);
                if self.current_page > page {
                    self.can_go_forward = true;
                } else {
                    self.can_go_forward = false;
                }

                if let Some(data) = self.carouselpages.get(&(page as usize)) {
                    match data {
                        StepType::Welcome => {
                            self.welcome.emit(WelcomeMsg::CheckSelected);
                        }
                        StepType::Keyboard => {
                            self.keyboard.emit(KeyboardMsg::CheckSelected);
                        }
                        StepType::Location => {
                            self.timezone.emit(TimeZoneMsg::CheckSelected);
                        }
                        StepType::Partitioning => {
                            self.partition.emit(PartitionMsg::CheckSelected);
                        }
                        StepType::User {
                            root: _,
                            hostname: _,
                        } => {
                            self.user.emit(UserMsg::CheckSelected);
                        }
                        StepType::Summary => {
                            self.summary.emit(SummaryMsg::SetConfig(
                                self.languageconfig.clone(),
                                self.keyboardconfig.clone(),
                                self.timezoneconfig.clone(),
                                self.partitionconfig.clone(),
                                Box::new(self.userconfig.clone()),
                            ));
                            self.can_go_forward = true;
                        }
                        StepType::List {
                            id: _,
                            multiple: _,
                            required,
                            title,
                            choices: _,
                        } => {
                            if *required {
                                if let Some(listpage) = self.list.get(title) {
                                    listpage.emit(ListMsg::CheckSelected)
                                } else {
                                    error!("List page not found: {}", title);
                                }
                            } else {
                                self.can_go_forward = true;
                            }
                        }
                        _ => {}
                    }
                }

                self.current_page = page;
            }
            AppMsg::SetCanGoBack(can_go_back) => {
                trace!("Carousel can go back: {}", can_go_back);
                self.can_go_back = can_go_back;
            }
            AppMsg::SetCanGoForward(can_go_forward) => {
                trace!("Carousel can go forward: {}", can_go_forward);
                self.can_go_forward = can_go_forward;
            }
            AppMsg::SetStackPage(page) => {
                debug!("StackPage: {:?}", page);
                if page == self.page {
                    return;
                }
                self.page = page;
            }
            AppMsg::SetStackPageConfig(page, installconfig) => {
                debug!("StackPage: {:?}", page);
                debug!("Config: {:?}", installconfig);
                if page == self.page {
                    return;
                }
                self.page = page;
                self.installconfig = installconfig;
                if let Some(cfg) = &self.installconfig {
                    let mut i = 0;
                    for step in &cfg.steps {
                        match step {
                            StepType::Welcome => {
                                trace!("Welcome append");
                                self.carousel.append(self.welcome.widget());
                                self.carouselpages.insert(i, StepType::Welcome);
                                i += 1;
                            }
                            StepType::Keyboard => {
                                trace!("Keyboard append");
                                self.carousel.append(self.keyboard.widget());
                                self.carouselpages.insert(i, StepType::Keyboard);
                                i += 1;
                            }
                            StepType::Location => {
                                trace!("Timezone append");
                                self.carousel.append(self.timezone.widget());
                                self.carouselpages.insert(i, StepType::Location);
                                i += 1;
                            }
                            StepType::Partitioning => {
                                trace!("Partitioning append");
                                self.carousel.append(self.partition.widget());
                                self.carouselpages.insert(i, StepType::Partitioning);
                                i += 1;
                            }
                            StepType::User { root, hostname } => {
                                trace!("User append");
                                self.carousel.append(self.user.widget());
                                self.carouselpages.insert(
                                    i,
                                    StepType::User {
                                        root: *root,
                                        hostname: *hostname,
                                    },
                                );
                                self.user.emit(UserMsg::SetConfig(
                                    if let Some(root) = root { *root } else { false },
                                    if let Some(hostname) = hostname {
                                        *hostname
                                    } else {
                                        false
                                    },
                                    self.config.default_hostname.to_string(),
                                ));
                                self.summary
                                    .emit(SummaryMsg::ShowHostname(hostname.unwrap_or(false)));
                                i += 1;
                            }
                            StepType::Summary => {
                                trace!("Summary append");
                                self.carousel.append(self.summary.widget());
                                self.carouselpages.insert(i, StepType::Summary);
                                i += 1;
                            }
                            StepType::List {
                                id,
                                multiple,
                                required,
                                title,
                                choices,
                            } => {
                                trace!("List append: {}", title);
                                let listpage = ListModel::builder()
                                    .launch(ListInit {
                                        id: id.to_string(),
                                        multiple: *multiple,
                                        required: *required,
                                        title: title.to_string(),
                                        choices: choices.clone(),
                                    })
                                    .forward(sender.input_sender(), identity);
                                self.carousel.append(listpage.widget());
                                self.list.insert(title.to_string(), listpage);
                                self.carouselpages.insert(
                                    i,
                                    StepType::List {
                                        id: id.to_string(),
                                        multiple: *multiple,
                                        required: *required,
                                        title: title.to_string(),
                                        choices: choices.clone(),
                                    },
                                );
                                self.listconfig.insert(id.to_string(), HashMap::new());
                                i += 1;
                            }
                            _ => {
                                warn!("Unimplemented step: {:?}", step);
                            }
                        }
                    }
                }
                sender.input(AppMsg::ChangePage(0));
            }
            AppMsg::SetLanguageConfig(language) => {
                self.languageconfig = language;
                for listpage in self.list.values() {
                    listpage.emit(ListMsg::SetLocale(self.languageconfig.clone()));
                }
                self.install
                    .emit(InstallMsg::SetLocale(self.languageconfig.clone()));
                if let Some(language) = &self.languageconfig {
                    if let (Ok(lang), Ok(country)) = (
                        get_lang(language.to_string()),
                        get_country(language.to_string()),
                    ) {
                        self.keyboard.emit(KeyboardMsg::SetCountry(lang, country));
                    }
                }
            }
            AppMsg::SetKeyboardConfig(keyboard) => {
                self.keyboardconfig = keyboard;
            }
            AppMsg::SetTimezoneConfig(timezone) => {
                self.timezoneconfig = timezone;
            }
            AppMsg::SetPartitionConfig(partition) => {
                self.partitionconfig = partition;
            }
            AppMsg::SetUserConfig(user) => {
                self.userconfig = user;
            }
            AppMsg::SetListConfig(title, list) => {
                info!("SetListConfig: {} {:?}", title, list);
                self.listconfig.insert(title, list);
                info!("ListConfig: {:?}", self.listconfig);
            }
            AppMsg::Install => {
                debug!("Installing!");
                if let Some(config) = &self.installconfig {
                    self.installworker.emit(InstallAsyncMsg::Install(
                        config.config_id.to_string(),
                        self.languageconfig.clone(),
                        self.timezoneconfig.clone(),
                        self.keyboardconfig.clone(),
                        Box::new(self.partitionconfig.clone()),
                        Box::new(self.userconfig.clone()),
                        self.listconfig.clone(),
                        config.config_type.clone(),
                        config.imperative_timezone.clone(),
                    ));
                }
            }
            AppMsg::FinishInstall => {
                debug!("Finishing install!");
                if let Some(config) = &self.installconfig {
                    self.installworker.emit(InstallAsyncMsg::FinishInstall(
                        self.timezoneconfig.clone(),
                        config.imperative_timezone.clone(),
                        config.commands.clone(),
                    ));
                }
            }
            AppMsg::RunNextCommand => {
                debug!("Running next postinstall command");
                self.installworker.emit(InstallAsyncMsg::RunNextCommand);
            }
            AppMsg::Finished => {
                debug!("Finished!");
                self.page = StackPage::Finished;
            }
            AppMsg::Error => {
                debug!("Error!");
                self.page = StackPage::Error;
                self.error.emit(ErrorMsg::Show);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppAsyncMsg::SetPage(page) => {
                self.page = page;
            }
        }
    }
}
