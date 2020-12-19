mod config;
mod find;
mod browser;

use fantoccini::Client;
use std::{thread, time};
use config::*;
use find::*;
use browser::*;
use std::collections::{HashSet};
use std::time::{Instant};
use std::error::Error;

#[tokio::main]
async fn main() {

    let config = match load_config("sites.toml") {
        Ok(val) => val,
        Err(err) => {
            println!("problem with config: {}",err);
            return;
        }
    };

    match process_groups(&config).await {
        Ok(_) => {
            println!("Ended OK");
        },
        Err(err) => { 
            println!("Ended in err {}", err);
        },
    };
}

async fn process_groups(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut start_time = Instant::now();
    let mut client = open_browser().await?;

    loop {
        if config.restart != 0 && start_time.elapsed().as_millis() as u64 >= config.restart {
            start_time = Instant::now();
            close_browser(&mut client).await?;
            client = open_browser().await?;
        }

        // client health check 
        client.refresh().await?;

        for group in &config.parallel_groups {
            match process_steps(group,&mut client).await {
                Ok(_) => {
                    // one of the groups ran to success, stop
                    println!("Group [{}] success, stopping", group.name);
                    return Ok(());
                },
                // ignore errors on process steps
                Err(_) => { },
            };
        }
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
                // if not in optional group, execute with error
                // if in non-failed optional group and fails, add to failed options
                // if in failed optional group, skip
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
            },
            Step::Wait(time) => {
                thread::sleep(time::Duration::from_millis(*time));
            },
        }
    }
    // all steps done
    Ok(())
}

fn log(message: String, log: &bool) {
    if *log {
        println!("{}",message);
    }
}