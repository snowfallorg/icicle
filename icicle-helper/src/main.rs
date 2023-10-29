use anyhow::{anyhow, Context, Result};
use clap::{self, FromArgMatches, Subcommand};
use disk_types::{BlockDeviceExt, FileSystem, PartitionTable, PartitionType, Sector, SectorExt};
use distinst_disk_ops::FormatPartitions;
use distinst_disks::{DiskExt, PartitionBuilder, PartitionFlag};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    process::Command,
};

#[derive(Serialize)]
struct Disk {
    name: String,
    size: u64,
    partitions: Vec<Partition>,
}

#[derive(Serialize)]
struct Partition {
    name: String,
    format: String,
    size: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub enum PartitionSchema {
    FullDisk(String),
    Custom(HashMap<String, CustomPartition>),
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomPartition {
    pub format: Option<String>,
    pub mountpoint: Option<String>,
    pub device: String,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    GetPartitions {},
    Partition {},
    WriteFile {
        #[clap(short, long)]
        path: String,
        #[clap(short, long)]
        contents: String,
    },
    Unmount {},
}

fn main() {
    let cli =
        SubCommands::augment_subcommands(clap::Command::new("Helper binary for Icicle installer"));
    let matches = cli.get_matches();
    let derived_subcommands = SubCommands::from_arg_matches(&matches)
        .map_err(|err| err.exit())
        .unwrap();

    if users::get_effective_uid() != 0 {
        eprintln!("icicle-helper must be run as root");
        std::process::exit(1);
    }

    match derived_subcommands {
        SubCommands::GetPartitions {} => {
            let mut outdisks = vec![];

            let mut devicevec = vec![];
            let devices = libparted::Device::devices(true);
            for device in devices {
                devicevec.push(device);
            }
            devicevec.sort_by(|a, b| a.path().to_str().cmp(&b.path().to_str()));
            for mut device in devicevec {
                let sectorsize = device.sector_size();
                let mut disk = Disk {
                    name: device.path().to_str().unwrap().to_string(),
                    size: device.length() * sectorsize,
                    partitions: vec![],
                };
                if let Ok(partdisk) = libparted::Disk::new(&mut device) {
                    let mut partvec = vec![];
                    for part in partdisk.parts() {
                        if part.get_path().is_none() {
                            continue;
                        }
                        partvec.push(part);
                    }
                    partvec.sort_by(|a, b| a.get_path().cmp(&b.get_path()));
                    for part in partvec {
                        disk.partitions.push(Partition {
                            name: part.get_path().unwrap().to_string_lossy().to_string(),
                            format: part.fs_type_name().unwrap_or("unknown").to_string(),
                            size: (part.geom_length() as u64) * sectorsize,
                        });
                    }
                }
                outdisks.push(disk);
            }
            println!("{}", serde_json::to_string(&outdisks).unwrap());
        }
        SubCommands::Partition {} => {
            partition().unwrap();
        }
        SubCommands::WriteFile { path, contents } => {
            fs::create_dir_all(path.rsplitn(2, '/').last().unwrap()).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        SubCommands::Unmount {} => {
            if let Err(e) = Command::new("umount")
                .arg("-R")
                .arg("-f")
                .arg("/tmp/icicle")
                .output()
            {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

fn partition() -> Result<()> {
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf)?;

    let schema: PartitionSchema = serde_json::from_str(&buf)?;

    match schema {
        PartitionSchema::FullDisk(diskpath) => {
            let start_sector = Sector::Start;
            let end_sector = Sector::End;
            let boot_sector = Sector::Unit(2_097_152);

            println!("Partition: Finding disk");
            let mut dev = distinst_disks::Disk::from_name(&diskpath)
                .ok()
                .ok_or_else(|| anyhow!("Failed to find disk"))?;
            let efi = distinst_disks::Bootloader::detect() == distinst_disks::Bootloader::Efi;

            if efi {
                println!("Partition: Creating GPT partition table");
                dev.mklabel(PartitionTable::Gpt)
                    .ok()
                    .ok_or_else(|| anyhow!("Failed to create GPT partition table"))?;

                println!("Partition: Creating EFI partition");
                // Add /boot partition
                dev.add_partition(
                    PartitionBuilder::new(
                        dev.get_sector(start_sector),
                        dev.get_sector(boot_sector),
                        FileSystem::Fat32,
                    )
                    .partition_type(PartitionType::Primary)
                    .flag(PartitionFlag::PED_PARTITION_ESP)
                    .mount("/boot".into()),
                )
                .ok()
                .ok_or_else(|| anyhow!("Failed to create EFI partition"))?;
            } else {
                println!("Partition: Creating MBR partition table");
                dev.mklabel(PartitionTable::Msdos)
                    .ok()
                    .ok_or_else(|| anyhow!("Failed to create MBR partition table"))?;
            }

            println!("Partition: Creating root partition");
            // Add root partition
            dev.add_partition(
                PartitionBuilder::new(
                    dev.get_sector(if efi { boot_sector } else { start_sector }),
                    dev.get_sector(end_sector),
                    FileSystem::Ext4,
                )
                .partition_type(PartitionType::Primary)
                .mount("/".into()),
            )
            .ok()
            .ok_or_else(|| anyhow!("Failed to create root partition"))?;

            println!("Partition: Committing changes");
            let partitions = dev
                .commit()
                .ok()
                .ok_or_else(|| anyhow!("Failed to commit changes"))?
                .context("Failed to get partitions")?;

            println!("Partition: Formatting partitions");
            let formatparts = FormatPartitions(partitions.0);
            formatparts
                .format()
                .ok()
                .ok_or_else(|| anyhow!("Failed to format partitions"))?;

            println!("Partition: Reloading disk");
            dev.reload()
                .ok()
                .ok_or_else(|| anyhow!("Failed to reload disk"))?;

            println!("Partition: Sorting partitions");
            let mut partvec = dev.get_partitions().to_vec();
            // Sort by shortest target first
            partvec.sort_by(|a, b| {
                a.target
                    .as_ref()
                    .map(|x| x.to_string_lossy().len())
                    .unwrap_or(0)
                    .cmp(
                        &b.target
                            .as_ref()
                            .map(|x| x.to_string_lossy().len())
                            .unwrap_or(0),
                    )
            });

            println!("Partition: Mounting partitions");
            for part in partvec {
                if let Some(target) = &part.target.as_ref().and_then(|x| x.to_str()) {
                    println!(" -- Target: {}", target);
                    println!(" -- Device: {}", part.get_device_path().to_string_lossy());
                    println!(
                        " -- Filesystem: {:?}",
                        part.filesystem.unwrap().to_string().as_str()
                    );
                    fs::create_dir_all(format!("/tmp/icicle{}", target))
                        .context("Failed to create mountpoint")?;
                    let output = if *target == "/boot" {
                        Command::new("mount")
                            .arg("-o")
                            .arg("umask=0077")
                            .arg(part.get_device_path())
                            .arg(format!("/tmp/icicle{}", target))
                            .output()
                            .context("Failed to mount partition")?
                    } else {
                        Command::new("mount")
                            .arg(part.get_device_path())
                            .arg(format!("/tmp/icicle{}", target))
                            .output()
                            .context("Failed to mount partition")?
                    };

                    if !output.status.success() {
                        return Err(anyhow!(
                            "Failed to mount partition: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
            }
        }
        PartitionSchema::Custom(partitions) => {
            let mut devices = HashMap::new();
            for (path, custom) in &partitions {
                if !devices.contains_key(&custom.device) {
                    let dev = distinst_disks::Disk::from_name(&custom.device)
                        .ok()
                        .ok_or_else(|| anyhow!("Failed to find disk {}", custom.device))?;
                    devices.insert(custom.device.to_string(), (dev, vec![]));
                }
                let partvec = &mut devices.get_mut(&custom.device).unwrap().1;
                partvec.push((path, custom));
                partvec.sort_by(|a, b| a.0.cmp(b.0));
            }

            // Loop through each modified disk
            for (device, (mut dev, partitions)) in devices {
                println!("Partitions: Partitioning disk {}", device);
                for (part, custom) in partitions {
                    let partition = dev
                        .partitions
                        .iter()
                        .find(|x| x.get_device_path().to_str() == Some(part))
                        .ok_or_else(|| anyhow!("Failed to find partition {}", part))?;
                    let num = &partition.number;
                    if let Some(format) = &custom.format.as_ref().and_then(|x| match x.as_str() {
                        "btrfs" => Some(FileSystem::Btrfs),
                        "ext4" => Some(FileSystem::Ext4),
                        "ext3" => Some(FileSystem::Ext3),
                        "fat32" => Some(FileSystem::Fat32),
                        "ntfs" => Some(FileSystem::Ntfs),
                        "xfs" => Some(FileSystem::Xfs),
                        "swap" => Some(FileSystem::Swap),
                        _ => None,
                    }) {
                        dev.format_partition(*num, *format)
                            .ok()
                            .ok_or_else(|| anyhow!("Failed to format partition {}", part))?;
                        if let Some(mountpoint) = &custom.mountpoint {
                            if mountpoint == "/boot" {
                                let partition = dev
                                    .partitions
                                    .iter_mut()
                                    .find(|x| x.get_device_path().to_str() == Some(part))
                                    .ok_or_else(|| anyhow!("Failed to find partition {}", part))?;
                                partition.flags.push(PartitionFlag::PED_PARTITION_ESP);
                            }
                        }
                    }
                }

                println!("Partitions: Committing changes");
                let parts = dev
                    .commit()
                    .ok()
                    .ok_or_else(|| anyhow!("Failed to commit changes to disk {}", device))?
                    .context("Failed to commit")?;

                println!("Partitions: Formatting partitions");
                let formatparts = FormatPartitions(parts.0);
                formatparts
                    .format()
                    .context("Failed to format partitions")?;
                dev.reload()
                    .ok()
                    .ok_or_else(|| anyhow!("Failed to reload disk {}", device))?;
            }

            println!("Partitions: Mounting partitions");
            let mut mountvec = partitions.into_iter().collect::<Vec<_>>();
            mountvec.sort_by(|a, b| {
                // Sort by mountpoint length, shortest first
                let a = a.1.mountpoint.as_ref().map(|x| x.len()).unwrap_or(0);
                let b = b.1.mountpoint.as_ref().map(|x| x.len()).unwrap_or(0);
                a.cmp(&b)
            });
            for (part, custom) in mountvec {
                if custom.format == Some("swap".to_string()) {
                    let _output = Command::new("swapon")
                        .arg(&part)
                        .output()
                        .context("Failed to enable swap")?;
                    continue;
                }
                if let Some(target) = custom.mountpoint {
                    fs::create_dir_all(format!("/tmp/icicle{}", target))
                        .context("Failed to create mountpoint")?;
                    let _output = if target == "/boot" {
                        Command::new("mount")
                        .arg("-o")
                        .arg("umask=0077")
                        .arg(&part)
                        .arg(format!("/tmp/icicle{}", target))
                        .output()
                        .context("Failed to mount partition")?
                    } else {
                        Command::new("mount")
                        .arg(&part)
                        .arg(format!("/tmp/icicle{}", target))
                        .output()
                        .context("Failed to mount partition")?
                    };
                }
            }
        }
    }
    Ok(())
}
