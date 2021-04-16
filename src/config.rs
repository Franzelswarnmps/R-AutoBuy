use serde_derive::Deserialize;
use std::error::Error;
use std::fs;

fn default_optional() -> bool { false }
fn default_logging() -> bool { true }
fn default_anti_cache() -> bool { false }
fn default_empty_string() -> String { "".into() }
fn default_wait() -> u64 { 0 }
fn default_delay() -> u64 { 0 }

#[derive(Debug, Deserialize)]
pub struct Step {
    #[serde(default = "default_empty_string")]
    pub name: String,
    pub action: StepAction,
    #[serde(default = "default_empty_string")]
    pub if_cond: String,
    #[serde(default = "default_empty_string")]
    pub if_not_cond: String,
    #[serde(default = "default_optional")]
    pub optional: bool,
    #[serde(default = "default_logging")]
    pub logging: bool,
    #[serde(default = "default_wait")]
    pub wait_max: u64,
    #[serde(default = "default_delay")]
    pub delay: u64,
}

#[derive(Debug, Deserialize)]
pub enum StepAction {
    Navigate{
        url: String,
        #[serde(default = "default_anti_cache")]
        anti_cache: bool,
    },
    Wait(u64),
    MatchUrl(String),
    Screenshot,
    TopWindow,
    Find{
        selector: String, 
        action: FindAction,
    },
    Refresh,
    End,
    Special(SpecialAction),
}

#[derive(Debug, Deserialize)]
pub enum SpecialAction {
    SolveAmazonReCaptcha
}

#[derive(Debug, Deserialize)]
pub enum FindAction {
    Click,
    Insert(String),
    SwitchFrame,
    None,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub profile: String,
    pub screenshot_path: String,
    pub marionette_port: u64,
    pub timeout: u64,
    pub groups: Vec<Group>,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let file_string = fs::read_to_string(path)?;
    Ok(toml::from_str(&file_string)?)
}