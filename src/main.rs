use adw::gio;
use gettextrs::{gettext, LocaleCategory};
use gtk::{glib, prelude::ApplicationExt};
use icicle::{
    config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE},
    ui::window::AppModel,
};
use log::{error, info};
use relm4::*;
use simplelog::*;
use std::fs::File;

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("/tmp/icicle.log").unwrap(),
        ),
    ])
    .unwrap();
    gtk::init().unwrap();
    setup_gettext();
    glib::set_application_name(&gettext("Icicle Installer"));
    if let Ok(res) = gio::Resource::load(RESOURCES_FILE) {
        info!("Resource loaded: {}", RESOURCES_FILE);
        gio::resources_register(&res);
    } else {
        error!("Failed to load resources");
    }
    gtk::Window::set_default_icon_name(icicle::config::APP_ID);
    let app = adw::Application::new(Some(icicle::config::APP_ID), gio::ApplicationFlags::empty());
    app.set_resource_base_path(Some("/org/snowflakeos/Icicle"));
    let app = RelmApp::from_app(app);
    app.run::<AppModel>(());
}

fn setup_gettext() {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to bind the text domain codeset to UTF-8");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");
}
