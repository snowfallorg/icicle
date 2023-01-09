use crate::{config::LIBEXECDIR, ui::window::AppMsg};
use adw::prelude::*;
use anyhow::{Context, Result};
use gettextrs::gettext;
use log::error;
use relm4::*;
use std::process::Command;
use tokio::io::AsyncWriteExt;

pub struct ErrorModel {
    messegebuffer: gtk::TextBuffer,
    uploadbutton: UploadButton,
    url: String,
    spinner: gtk::Spinner,
}

#[derive(Debug)]
pub enum UploadButton {
    Button,
    Loading,
    Url,
}

#[derive(Debug)]
pub enum ErrorMsg {
    Show,
    UploadReport,
    SetUrl(String),
    SetUploadButton(UploadButton),
}

#[relm4::component(pub)]
impl SimpleComponent for ErrorModel {
    type Init = ();
    type Input = ErrorMsg;
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
                        set_label: &gettext("Installation Failed!"),
                    },
                    gtk::Image {
                        add_css_class: "error",
                        set_icon_name: Some("process-stop-symbolic"),
                        set_pixel_size: 128,
                    },
                    gtk::Frame {
                        gtk::ScrolledWindow {
                            set_height_request: 300,
                            gtk::TextView {
                                set_editable: false,
                                set_hexpand: true,
                                set_vexpand: true,
                                set_buffer: Some(&model.messegebuffer),
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                set_right_margin: 5,
                                set_monospace: true,
                            }
                        }
                    },
                    match &model.uploadbutton {
                        UploadButton::Button => {
                            gtk::Button {
                                add_css_class: "pill",
                                set_halign: gtk::Align::Center,
                                #[watch]
                                set_label: &gettext("Upload Report"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(ErrorMsg::UploadReport);
                                }
                            }
                        },
                        UploadButton::Loading => {
                            #[local]
                            spinner -> gtk::Spinner {
                                set_spinning: true,
                                set_halign: gtk::Align::Center,
                                set_size_request: (48, 48),
                            }
                        },
                        UploadButton::Url => {
                            gtk::LinkButton {
                                set_css_classes: &["pill", "suggested-action"],
                                #[watch]
                                set_label: &gettext("Open Report"),
                                #[watch]
                                set_uri: &model.url,
                                set_halign: gtk::Align::Center,
                            }
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
        let model = ErrorModel {
            uploadbutton: UploadButton::Button,
            url: String::new(),
            messegebuffer: gtk::TextBuffer::new(None),
            spinner: gtk::Spinner::new(),
        };
        let spinner = model.spinner.clone();
        let widgets = view_output!();
        sender.input(ErrorMsg::Show);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            ErrorMsg::Show => {
                if let Err(e) = Command::new("pkexec")
                    .arg(&format!("{}/icicle-helper", LIBEXECDIR))
                    .arg("unmount")
                    .output()
                {
                    error!("Failed to unmount partitions: {}", e);
                }

                let mut outlog = "=== Icicle Log ===\n".to_string();
                if let Ok(iciclelog) = std::fs::read_to_string("/tmp/icicle.log") {
                    outlog.push_str(iciclelog.trim());
                } else {
                    outlog.push_str("No log found!");
                }
                outlog.push_str("\n=== End of Icicle Log ===\n\n");
                outlog.push_str("=== nixos-install Log ===\n");
                if let Ok(nixoslog) = std::fs::read_to_string("/tmp/icicle-term.log") {
                    outlog.push_str(nixoslog.trim());
                } else {
                    outlog.push_str("No log found!");
                }
                outlog.push_str("\n=== End of nixos-install Log ===\n\n");
                self.messegebuffer.set_text(&outlog);
            }
            ErrorMsg::UploadReport => {
                let text = self
                    .messegebuffer
                    .text(
                        &self.messegebuffer.start_iter(),
                        &self.messegebuffer.end_iter(),
                        true,
                    )
                    .to_string();
                self.uploadbutton = UploadButton::Loading;
                self.spinner.set_spinning(false);
                self.spinner.activate();
                self.spinner.set_spinning(true);
                relm4::spawn(async move {
                    async fn termbin(text: String) -> Result<String> {
                        let mut response = tokio::process::Command::new("nc")
                            .arg("termbin.com")
                            .arg("9999")
                            .stdin(std::process::Stdio::piped())
                            .stdout(std::process::Stdio::piped())
                            .spawn()?;

                        response
                            .stdin
                            .take()
                            .context("Failed to get stdin")?
                            .write_all(text.as_bytes())
                            .await
                            .context("Failed to write to stdin")?;
                        let url = String::from_utf8(
                            response
                                .wait_with_output()
                                .await
                                .context("Failed to get output")?
                                .stdout,
                        )
                        .context("Failed to read stdout")?;
                        Ok(url)
                    }
                    match termbin(text).await {
                        Ok(url) => {
                            sender.input(ErrorMsg::SetUrl(url.trim().replace(['\0', '\n'], "")));
                        }
                        Err(e) => {
                            error!("Failed to upload report: {}", e);
                            sender.input(ErrorMsg::SetUploadButton(UploadButton::Button));
                        }
                    }
                });
            }
            ErrorMsg::SetUrl(url) => {
                self.url = url;
                self.uploadbutton = UploadButton::Url;
            }
            ErrorMsg::SetUploadButton(button) => {
                self.uploadbutton = button;
            }
        }
    }
}
