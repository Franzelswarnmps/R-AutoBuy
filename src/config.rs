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
    Find{
        name: String, 
        selector: String, 
        #[serde(default = "default_optional")]
        optional_group: String, 
        action: FindAction,
    },
    End,
}

fn default_optional() -> String {
    "".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ParallelGroup {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub interval: u64,
    pub parallel_groups: Vec<ParallelGroup>,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let file_string = fs::read_to_string(path)?;
    match toml::from_str(&file_string) {
        Ok(val) => Ok(val),
        Err(err) => Err(Box::new(err))
    }
}