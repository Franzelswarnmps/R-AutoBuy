use serde_derive::Deserialize;
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize)]
pub enum Compare {
    //Less(String),
    Equal(String),
    //Greater(String),
    //Between{min: String, max: String},
}

#[derive(Debug, Deserialize)]
pub enum FindAction {
    //Read,
    Click,
    Insert(String),
    Compare(Compare),
    None,
}

#[derive(Debug, Deserialize)]
pub enum Step {
    Navigate(String),
    Log(String),
    Wait(u64),
    Find{
        name: String, 
        selector: String, 
        action: FindAction,
        #[serde(default = "default_optional")]
        optional_group: String, 
        #[serde(default = "default_wait")]
        wait_max: u64,
        #[serde(default = "default_delay")]
        delay: u64,
        #[serde(default = "default_logging")]
        logging: bool, 
    },
}

fn default_optional() -> String { "".to_string() }
fn default_wait() -> u64 { 0 }
fn default_delay() -> u64 { 0 }
fn default_logging() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct ParallelGroup {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub restart: u64,
    pub parallel_groups: Vec<ParallelGroup>,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let file_string = fs::read_to_string(path)?;
    match toml::from_str(&file_string) {
        Ok(val) => Ok(val),
        Err(err) => Err(Box::new(err))
    }
}