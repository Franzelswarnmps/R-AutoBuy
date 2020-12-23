use serde_derive::Deserialize;
use std::error::Error;
use std::fs;
use std::collections::{HashMap};


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
    Refresh,
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
    pub timeout: u64,
    pub parallel_groups: Vec<ParallelGroup>,
}

#[derive(Debug, Deserialize)]
pub struct Secrets {
    pub pairs: HashMap<String, String>,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let file_string = fs::read_to_string(path)?;
    match toml::from_str(&file_string) {
        Ok(val) => Ok(val),
        Err(err) => Err(Box::new(err))
    }
}

pub fn load_secrets(path: &str) -> Result<Secrets, Box<dyn Error>> {
    let file_string = fs::read_to_string(path)?;
    match toml::from_str(&file_string) {
        Ok(val) => Ok(val),
        Err(err) => Err(Box::new(err))
    }
}

pub fn populate_secrets(config: &mut Config, secrets: &Secrets) {
    for group in &mut config.parallel_groups {
        for step in &mut group.steps {
            match step {
               Step::Find{action, ..} => {
                match action {
                    FindAction::Insert(val) => {
                        match secrets.pairs.get(val) {
                            Some(secret) => {
                                *val = secret.to_string();
                            },
                            None => {},
                        }
                    },
                    _ => {},
                }
               },
                _ => {},
            }
        }
    }
}