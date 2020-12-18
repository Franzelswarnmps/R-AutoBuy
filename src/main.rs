mod config;
mod find;

use fantoccini::Client;
use std::process::Command;
use std::{thread, time};
use config::*;
use find::*;
use std::collections::{HashSet};
// use serde_json::map::Map;
// use serde_json::value::Value;
// use serde_json::json;


#[tokio::main]
async fn main() {
    // tasklist /FI "IMAGENAME eq myapp.exe" 2>NUL | find /I /N "myapp.exe">NUL

    let config = match load_config("sites.toml") {
        Ok(val) => val,
        Err(err) => {
            println!("problem with config: {}",err);
            return;
        }
    };

    Command::new(".\\geckodriver.exe").spawn().expect("failed to start geckodriver.exe");

    // let mut capabilities: Map<String,Value> = Map::new();
    // capabilities.insert("permissions.default.image".to_string(),json!(2));
    // let mut client = Client::with_capabilities("http://localhost:4444",capabilities).await.expect("failed to connect to WebDriver");
    let mut client = Client::new("http://localhost:4444").await.expect("failed to connect to WebDriver");

    match process_groups(config, &mut client).await {
        Ok(_) => {println!("ended OK")},
        Err(err) => {println!("ended ERR: {}",err)}
    }
     match client.close().await {
        Ok(_) => {},
        Err(err) => {println!("client could not be closed: {}",err)}
    }

    Command::new("taskkill")
    .args(&["/f", "/im", "geckodriver.exe"])
    .output()
    .expect("failed to stop geckodriver.exe");
}

async fn process_groups(config: Config, client: &mut Client) -> Result<(), fantoccini::error::CmdError> {

    loop {
        for group in &config.parallel_groups {
            match process_steps(group,client).await {
                Ok(_) => {
                    // one of the groups ran to success, stop
                    println!("Group [{}] success, stopping", group.name);
                    return Ok(());
                },
                Err(_) => {},
            };

            // client.new_window(true).await?;
            // let window = client.windows().await?.last().unwrap().clone();

            // client.close_window().await?;
            // client.switch_to_window(window).await?;
        }

        thread::sleep(time::Duration::from_secs(config.interval));
    }
}

async fn process_steps(group: &ParallelGroup, c: &mut Client) -> Result<(), fantoccini::error::CmdError> {
    let mut failed_options = HashSet::new();

    for step in &group.steps {
        //println!("starting step {}", step.name);
        match step {
            Step::Navigate(dest) => {
                c.goto(dest).await?;
            },
            Step::Find{name, selector, action, optional_group, wait_max, delay, logging} => {
                // if not in optionary group, execute with error
                // if in non-failed optionary group and fails, add to failed options
                // if in failed optionary group, skip
                match failed_options.get(optional_group) {
                    None => {
                        // println!("Step [{}] starting", name);
                        match process_find(c,selector, action, wait_max, delay).await {
                            Ok(_) => {
                                log(format!("Step [{}:{}] success", group.name, name),logging);
                            },
                            Err(err) => {
                                if optional_group != "" {
                                    log(format!("Step [{}:{}] option [{}] failed: {}", group.name, name, optional_group, err),logging);
                                    failed_options.insert(optional_group);
                                } else {
                                    log(format!("Step [{}:{}] failed, restarting: {}", group.name, name, err),logging);
                                    return Result::Err(err);
                                }
                            }
                        };
                    },
                    Some(_) => {
                        log(format!("Optional step [{}:{}] skipped", group.name, name),logging);
                        continue;
                    }
                }
            },
            Step::Log(message) => {
                println!("{}",message);
            }
        }
    }
    Ok(())
}

fn log(message: String, log: &bool) {
    if *log {
        println!("{}",message);
    }
}