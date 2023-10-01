use crate::config::SYSCONFDIR;
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IcicleConfig {
    pub distribution_name: String,
    pub branding: String,
    pub internet_check_url: String,
    pub default_hostname: String,
    pub choices: Vec<ChoiceEnum>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ChoiceEnum {
    Configuration {
        file: String,
        #[serde(skip)]
        config: InstallationConfig,
    },
    Live,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigType {
    Snowfall,
    #[default]
    Flakes,
    Legacy
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct InstallationConfig {
    pub config_id: String,
    pub config_name: String,
    pub config_logo: String,
    pub config_type: ConfigType,
    #[serde(default)]
    pub imperative_timezone: bool,
    pub steps: Vec<StepType>,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    Welcome,
    Location,
    Keyboard,
    User {
        root: Option<bool>,
        hostname: Option<bool>,
    },
    List {
        id: String,
        multiple: bool,
        required: bool,
        title: String,
        choices: Vec<HashMap<String, Choice>>,
    },
    Partitioning,
    Manual,
    Summary,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Choice {
    pub description: Option<String>,
    pub packages: Option<Vec<String>>,
    pub config: Option<String>,
}

pub fn parse_config() -> Result<IcicleConfig> {
    debug!("Parsing config {}/icicle/config.yml", SYSCONFDIR);
    let f = fs::read_to_string(&format!("{}/icicle/config.yml", SYSCONFDIR))?;
    let mut config: IcicleConfig = serde_yaml::from_str(&f)?;
    for choice in &mut config.choices {
        match choice {
            ChoiceEnum::Configuration { file, config } => {
                let f = fs::read_to_string(&format!("{}/icicle/{}", SYSCONFDIR, file))?;
                *config = serde_yaml::from_str(&f)?;
            }
            ChoiceEnum::Live => {}
        }
    }
    Ok(config)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BrandingConfig {
    pub slides: Vec<Slide>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Slide {
    pub title: String,
    pub subtitle: String,
    pub image: String,
}

pub fn parse_branding(brand: &str) -> Result<BrandingConfig> {
    let f = fs::read_to_string(&format!(
        "{}/icicle/branding/{}/slides.yml",
        SYSCONFDIR, brand
    ))?;
    let config: BrandingConfig = serde_yaml::from_str(&f)?;
    Ok(config)
}
