use crate::{config::LIBEXECDIR, ui::window::AppMsg, utils::i18n::i18n_f};
use adw::prelude::*;
use gettextrs::gettext;
use log::{debug, error, info, trace};
use relm4::{factory::*, *};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};

pub struct PartitionModel {
    disks: FactoryVecDeque<WholeDisk>,
    method: PartitionMethod,
    partition_groups: FactoryVecDeque<PartitionGroup>,
    diskgroupbtn: gtk::CheckButton,
    schema: Option<PartitionSchema>,
    efi: bool,
}

#[derive(Debug)]
pub enum PartitionMsg {
    SetMethod(PartitionMethod),
    SetFullDisk(PartitionSchema),
    AddFormatPartition(String, String, String),
    AddMountPartition(String, String, String),
    RemoveFormatPartition(String),
    RemoveMountPartition(String),
    AddPartition(String, CustomPartition),
    CheckSelected,
    Refresh,
}

pub static PARTITION_BROKER: MessageBroker<PartitionMsg> = MessageBroker::new();

#[derive(Debug, PartialEq, Eq)]
pub enum PartitionMethod {
    Basic,
    Advanced,
}

#[derive(Serialize, Debug, Clone)]
pub enum PartitionSchema {
    FullDisk(String),
    Custom(HashMap<String, CustomPartition>),
}

#[derive(Serialize, Debug, Clone)]
pub struct CustomPartition {
    pub format: Option<String>,
    pub mountpoint: Option<String>,
    pub device: String,
}

#[relm4::component(pub)]
impl SimpleComponent for PartitionModel {
    type Input = PartitionMsg;
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
                    set_margin_start: 30,
                    set_margin_end: 30,
                    set_margin_top: 20,
                    set_margin_bottom: 20,
                    #[name(liststack)]
                    match model.method {
                        PartitionMethod::Basic => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Select a disk"),
                                add_css_class: "title-1"
                            },
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Disk will be formatted and all data will be lost"),
                                add_css_class: "dim-label",
                                add_css_class: "title-3"
                            },
                            #[local_ref]
                            diskbox -> gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_hexpand: true,
                                set_selection_mode: gtk::SelectionMode::None,
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 20,
                                set_halign: gtk::Align::Center,
                                gtk::Button {
                                    add_css_class: "pill",
                                    #[watch]
                                    set_label: &gettext("Advanced"),
                                    set_halign: gtk::Align::Center,
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::SetMethod(PartitionMethod::Advanced));
                                    }
                                },
                                gtk::Button {
                                    set_valign: gtk::Align::Center,
                                    add_css_class: "pill",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::Refresh);
                                    },
                                    adw::ButtonContent {
                                        set_icon_name: "view-refresh-symbolic",
                                        #[watch]
                                        set_label: &gettext("Refresh")
                                    }
                                }
                            }
                        },
                        PartitionMethod::Advanced => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Select partitions"),
                                add_css_class: "title-1"
                            },

                            gtk::Button {
                                #[watch]
                                set_css_classes: if let Some(PartitionSchema::Custom(schema)) = &model.schema {
                                    let mut root = false;
                                    let mut bootefi = !model.efi;
                                    for v in schema.values() {
                                        if let Some(file) = &v.mountpoint {
                                            if file == "/" {
                                                root = true;
                                            }
                                            if file == "/boot" {
                                                bootefi = true;
                                            }
                                        }
                                    }
                                    match (root, bootefi) {
                                        (true, true) => &["pill", "success"],
                                        (true, false) => &["pill", "error"],
                                        (false, true) => &["pill", "error"],
                                        (false, false) => &["pill", "error"],
                                    }
                                } else {
                                    &["pill", "error"]
                                },
                                set_can_target: false,
                                gtk::Label {
                                    #[watch]
                                    set_markup: &if let Some(PartitionSchema::Custom(schema)) = &model.schema {
                                        let mut root = false;
                                        let mut bootefi = !model.efi;
                                        for v in schema.values() {
                                            if let Some(file) = &v.mountpoint {
                                                if file == "/" {
                                                    root = true;
                                                }
                                                if file == "/boot" {
                                                    bootefi = true;
                                                }
                                            }
                                        }
                                        match (root, bootefi) {
                                            (true, true) => gettext("Ready to install!"),
                                            // Translators: Do NOT translate anything between the <tt> tags
                                            (true, false) => gettext("Missing <tt>/boot</tt> partition"),
                                            // Translators: Do NOT translate anything between the <tt> tags
                                            (false, true) => gettext("Missing <tt>/</tt> partition"),
                                            // Translators: Do NOT translate anything between the <tt> tags
                                            (false, false) => gettext("Missing <tt>/</tt> and <tt>/boot</tt> partitions"),
                                        }
                                    } else if model.efi {
                                        // Translators: Do NOT translate anything between the <tt> tags
                                        gettext("Missing <tt>/</tt> and <tt>/boot</tt> partitions")
                                    } else {
                                        gettext("Missing <tt>/</tt> partition")
                                    }
                                }
                            },

                            #[local_ref]
                            partitionbox -> gtk::Box {
                                set_hexpand: true,
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 20,
                            },
                            gtk::Button {
                                add_css_class: "pill",
                                adw::ButtonContent {
                                    set_icon_name: "drive-multidisk-symbolic",
                                    #[watch]
                                    set_label: &gettext("Launch GParted"),
                                },
                                set_halign: gtk::Align::Center,
                                connect_clicked => move |_| {
                                    let cmd = Command::new("gparted").spawn();
                                    if let Err(e) = cmd {
                                        error!("Failed to launch GParted: {}", e);
                                    }
                                }
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 20,
                                set_halign: gtk::Align::Center,
                                gtk::Button {
                                    add_css_class: "pill",
                                    #[watch]
                                    set_label: &gettext("Basic"),
                                    set_halign: gtk::Align::Center,
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::SetMethod(PartitionMethod::Basic));
                                    }
                                },
                                gtk::Button {
                                    set_valign: gtk::Align::Center,
                                    add_css_class: "pill",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::Refresh);
                                    },
                                    adw::ButtonContent {
                                        set_icon_name: "view-refresh-symbolic",
                                        #[watch]
                                        set_label: &gettext("Refresh")
                                    }
                                }
                            }
                        }
                    },

                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PartitionModel {
            disks: FactoryVecDeque::builder()
                .launch_default()
                .detach(),
            method: PartitionMethod::Basic,
            partition_groups: FactoryVecDeque::builder()
            .launch(gtk::Box::new(
                gtk::Orientation::Vertical,
                20,
            ))
            .detach(),
            diskgroupbtn: gtk::CheckButton::new(),
            schema: None,
            efi: distinst_disks::Bootloader::detect() == distinst_disks::Bootloader::Efi,
        };

        sender.input(PartitionMsg::Refresh);

        let diskbox = model.disks.widget();
        let partitionbox = model.partition_groups.widget();

        let widgets = view_output!();
        widgets.liststack.set_vhomogeneous(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            PartitionMsg::Refresh => {
                let mut disks_guard = self.disks.guard();
                let mut partition_groups_guard = self.partition_groups.guard();

                disks_guard.clear();
                partition_groups_guard.clear();

                let out = Command::new("pkexec")
                    .arg(&format!("{}/icicle-helper", LIBEXECDIR))
                    .arg("get-partitions")
                    .output();

                match out {
                    Ok(out) => {
                        let output = String::from_utf8(out.stdout).unwrap();
                        let stderr = String::from_utf8(out.stderr).unwrap();
                        #[derive(Deserialize, Debug)]
                        struct InputDisk {
                            name: String,
                            size: u64,
                            partitions: Vec<InputPartition>,
                        }

                        #[derive(Deserialize, Debug)]
                        struct InputPartition {
                            name: String,
                            #[serde(rename = "format")]
                            _format: String,
                            size: u64,
                        }
                        let disks: serde_json::Result<Vec<InputDisk>> =
                            serde_json::from_str(&output);
                        if let Ok(disks) = disks {
                            debug!("Got disks: {:?}", disks);

                            for disk in disks {
                                disks_guard.push_back(WholeDisk {
                                    name: disk.name.to_string(),
                                    size: disk.size,
                                    group: self.diskgroupbtn.clone(),
                                });

                                let mut part_factoryvec: FactoryVecDeque<Partition> =
                                    FactoryVecDeque::builder()
                                        .launch_default()
                                        .detach();
                                let mut part_guard = part_factoryvec.guard();

                                for part in disk.partitions {
                                    info!(
                                        "Partition: {:?} length {}",
                                        part.name,
                                        size::Size::from_bytes(part.size)
                                    );
                                    part_guard.push_back(PartitionInit {
                                        name: part.name,
                                        size: part.size,
                                        mountrow: adw::ComboRow::new(),
                                        device: disk.name.to_string(),
                                    });
                                }

                                part_guard.drop();

                                partition_groups_guard.push_back(PartitionGroup {
                                    name: disk.name.to_string(),
                                    partitions: part_factoryvec,
                                });
                            }
                        } else {
                            error!("Failed to parse partitions: {} : {}", output, stderr);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get partitions: {}", e);
                    }
                }

                disks_guard.drop();
                partition_groups_guard.drop();
                self.schema = None;
            }
            PartitionMsg::SetMethod(method) => {
                self.method = method;
                self.schema = None;
                self.diskgroupbtn.set_active(true);
                let _ = sender.output(AppMsg::SetCanGoForward(false));
                sender.input(PartitionMsg::Refresh);
            }
            PartitionMsg::SetFullDisk(schema) => {
                trace!("SetFullDisk");
                self.schema = Some(schema);
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::AddFormatPartition(name, format, device) => {
                trace!("AddFormatPartition");
                if let Some(PartitionSchema::Custom(schema)) = &mut self.schema {
                    if let Some(part) = &mut schema.get_mut(&name) {
                        part.format = Some(format);
                    } else {
                        schema.insert(
                            name,
                            CustomPartition {
                                format: Some(format),
                                mountpoint: None,
                                device,
                            },
                        );
                    }
                } else {
                    let mut schema = HashMap::new();
                    schema.insert(
                        name,
                        CustomPartition {
                            format: Some(format),
                            mountpoint: None,
                            device,
                        },
                    );
                    self.schema = Some(PartitionSchema::Custom(schema));
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::AddMountPartition(name, mount, device) => {
                trace!("AddMountPartition");
                if let Some(PartitionSchema::Custom(schema)) = &mut self.schema {
                    // Check if the mountpoint is already in use
                    for part in schema.values() {
                        if let Some(partmount) = &part.mountpoint {
                            if partmount == &mount {
                                let mut partition_group_guard = self.partition_groups.guard();
                                for i in 0..partition_group_guard.len() {
                                    let partition_guard =
                                        partition_group_guard[i].partitions.guard();
                                    for i in 0..partition_guard.len() {
                                        if partition_guard[i].name != name {
                                            trace!(
                                                "Deselecting {} {}",
                                                partition_guard[i].name,
                                                mount
                                            );
                                            partition_guard.send(
                                                i,
                                                PartitionRowMsg::Deselect(mount.to_string()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(part) = &mut schema.get_mut(&name) {
                        part.mountpoint = Some(mount);
                    } else {
                        schema.insert(
                            name,
                            CustomPartition {
                                format: None,
                                mountpoint: Some(mount),
                                device,
                            },
                        );
                    }
                } else {
                    let mut schema = HashMap::new();
                    schema.insert(
                        name,
                        CustomPartition {
                            format: None,
                            mountpoint: Some(mount),
                            device,
                        },
                    );
                    self.schema = Some(PartitionSchema::Custom(schema));
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::RemoveFormatPartition(name) => {
                trace!("RemoveFormatPartition");
                if let Some(PartitionSchema::Custom(schema)) = &mut self.schema {
                    if let Some(part) = &mut schema.get_mut(&name) {
                        if part.mountpoint.is_none() {
                            schema.remove(&name);
                        } else {
                            part.format = None;
                        }
                    }
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::RemoveMountPartition(name) => {
                trace!("RemoveMountPartition");
                if let Some(PartitionSchema::Custom(schema)) = &mut self.schema {
                    if let Some(part) = &mut schema.get_mut(&name) {
                        if part.format.is_none() {
                            schema.remove(&name);
                        } else {
                            part.mountpoint = None;
                        }
                    }
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::AddPartition(name, format) => {
                trace!("AddPartition");
                if let Some(PartitionSchema::Custom(schema)) = &mut self.schema {
                    schema.insert(name, format);
                } else {
                    let mut schema = HashMap::new();
                    schema.insert(name, format);
                    self.schema = Some(PartitionSchema::Custom(schema));
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::CheckSelected => {
                trace!("PartitionMsg::CheckSelected: {:?}", self.schema);
                match &self.schema {
                    Some(PartitionSchema::FullDisk(_disk)) => {
                        let _ = sender.output(AppMsg::SetCanGoForward(true));
                        let _ = sender.output(AppMsg::SetPartitionConfig(self.schema.clone()));
                    }
                    Some(PartitionSchema::Custom(schema)) => {
                        let mut root = false;
                        let mut bootefi = false;
                        for part in schema.values() {
                            if part.mountpoint == Some("/".to_string()) {
                                root = true;
                            }
                            if part.mountpoint == Some("/boot".to_string()) || !self.efi {
                                bootefi = true;
                            }
                        }
                        let _ = sender.output(AppMsg::SetCanGoForward(root && bootefi));
                        if root && bootefi {
                            let _ = sender.output(AppMsg::SetPartitionConfig(self.schema.clone()));
                        }
                    }
                    None => {
                        let _ = sender.output(AppMsg::SetCanGoForward(false));
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct WholeDisk {
    name: String,
    size: u64,
    group: gtk::CheckButton,
}

#[relm4::factory(pub)]
impl FactoryComponent for WholeDisk {
    type Init = WholeDisk;
    type Input = ();
    type Output = ();
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            set_title: &self.name,
            #[watch]
            // Translators: Do NOT translate the '{}'
            // The string reads "{/dev/sdX} (20 GB minimum needed)" indicating that the given disk is not large enough
            set_subtitle: &if self.size > 21_474_836_480  { size::Size::from_bytes(self.size).to_string() } else { i18n_f("{} (20 GB minimum needed)", &[&size::Size::from_bytes(self.size).to_string()]) },
            set_activatable: true,
            set_sensitive: self.size > 21_474_836_480, // 20GB
            #[name(checkbtn)]
            add_suffix = &gtk::CheckButton {
                set_group: Some(&self.group),
                connect_toggled[name = self.name.to_string()] => move |btn| {
                    if btn.is_active() {
                        PARTITION_BROKER.send(PartitionMsg::SetFullDisk(PartitionSchema::FullDisk(name.to_string())));
                    }
                }
            },
            connect_activated[checkbtn, name = self.name.to_string()] => move |_| {
                checkbtn.set_active(true);
                PARTITION_BROKER.send(PartitionMsg::SetFullDisk(PartitionSchema::FullDisk(name.to_string())));
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Partition {
    name: String,
    size: u64,
    mountrow: adw::ComboRow,
    device: String,
    swap: bool,
    donotmount: String,
    donotformat: String,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PartitionInit {
    name: String,
    size: u64,
    mountrow: adw::ComboRow,
    device: String,
}

#[derive(Debug)]
pub enum PartitionRowMsg {
    Deselect(String),
    SetSwap(bool),
}

#[relm4::factory(pub)]
impl FactoryComponent for Partition {
    type Init = PartitionInit;
    type Input = PartitionRowMsg;
    type Output = ();
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();

    view! {
        adw::ExpanderRow {
            set_title: &self.name,
            set_subtitle: &size::Size::from_bytes(self.size).to_string(),
            add_row = &adw::ComboRow {
                #[watch]
                set_title: &gettext("Format"),
                // TODO: When switching language the "Leave as is" option does not update
                set_model: Some(&gtk::StringList::new(&[&self.donotformat, "btrfs", "ext4", "ext3", "fat32", "ntfs", "xfs", "swap"])),
                connect_selected_notify[sender, name = self.name.to_string(), device = self.device.to_string(), formatstring = self.donotformat.to_string()] => move |row| {
                    if let Some(item) = row.selected_item() {
                        if let Ok(item) = item.downcast::<gtk::StringObject>() {
                            if item.string() == formatstring {
                                PARTITION_BROKER.send(PartitionMsg::RemoveFormatPartition(name.to_string()));
                            } else {
                                PARTITION_BROKER.send(PartitionMsg::AddFormatPartition(name.to_string(), item.string().to_string(), device.to_string()));
                            }
                            sender.input(PartitionRowMsg::SetSwap(item.string().eq("swap")));
                        }
                    }
                }
            },
            #[local_ref]
            add_row = mountrow -> adw::ComboRow {
                #[watch]
                set_visible: !self.swap,
                #[watch]
                set_title: &gettext("Mount"),
                // TODO: When switching language the "Do not mount" option does not update
                set_model: Some(&gtk::StringList::new(&[&self.donotmount, " /", "/boot", "/home", "/opt", "/var", "/nix"])),
                connect_selected_notify[name = self.name.to_string(), device = self.device.to_string(), mountstring = self.donotmount.to_string()] => move |row| {
                    if let Some(item) = row.selected_item() {
                        if let Ok(item) = item.downcast::<gtk::StringObject>() {
                            if item.string() == mountstring {
                                PARTITION_BROKER.send(PartitionMsg::RemoveMountPartition(name.to_string()));
                            } else {
                                PARTITION_BROKER.send(PartitionMsg::AddMountPartition(name.to_string(), item.string().trim().to_string(), device.to_string()));
                            }
                        }
                    }
                }
            },
            add_row = &adw::ActionRow {
                #[watch]
                set_visible: self.swap,
                #[watch]
                set_title: &gettext("Mount"),
                add_suffix = &gtk::Label {
                    set_text: "swap",
                }
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Partition {
            name: parent.name,
            size: parent.size,
            mountrow: parent.mountrow,
            device: parent.device,
            swap: false,
            donotmount: gettext("Do not mount"),
            donotformat: gettext("Leave as is"),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let mountrow = &self.mountrow;
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            PartitionRowMsg::Deselect(mount) => {
                if let Some(item) = self.mountrow.selected_item() {
                    if let Ok(item) = item.downcast::<gtk::StringObject>() {
                        if item.string().eq(&mount) {
                            self.mountrow.set_selected(0);
                        }
                    }
                }
            }
            PartitionRowMsg::SetSwap(swap) => {
                self.swap = swap;
            }
        }
    }
}

pub struct PartitionGroup {
    name: String,
    partitions: FactoryVecDeque<Partition>,
}

#[relm4::factory(pub)]
impl FactoryComponent for PartitionGroup {
    type Init = PartitionGroup;
    type Input = ();
    type Output = ();
    type ParentWidget = gtk::Box;
    type CommandOutput = ();

    view! {
        adw::PreferencesGroup {
            set_title: &self.name,
            #[local_ref]
            testbox -> gtk::ListBox {
                add_css_class: "boxed-list",
                set_hexpand: true,
                set_selection_mode: gtk::SelectionMode::None,
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let testbox = self.partitions.widget();
        let widgets = view_output!();
        widgets
    }
}
