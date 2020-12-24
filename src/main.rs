mod config;
mod find;
mod browser;

use std::{thread, time};
use config::*;
use find::*;
use browser::*;
use std::collections::{HashSet};
//use std::time::{Instant};
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut config = load_config("sites.toml")?;
    let secrets = load_secrets("secrets.toml")?;
    populate_secrets(&mut config, &secrets);

    let mut browser = Browser::new(config.groups.len(),Duration::from_millis(config.timeout)).await?;

    match process_groups(&config, &mut browser).await {
        Ok(_) => {
            println!("Ended OK");
        },
        Err(err) => { 
            println!("Ended in err {}", err);
            browser.close().await?
        },
    };
    Ok(())
}

async fn process_groups(config: &Config, browser: &mut Browser) -> Result<(), Box<dyn Error>> {
    // let mut start_time = Instant::now();

    let mut startup_executed = vec![false;config.groups.len()];
    loop {
        for (index,group) in config.groups.iter().enumerate() {

            // switch to correct tab
            match browser.switch_tab(index).await {
                Err(err) => {
                    println!("Group [{}] tab switch error {}, restarting", group.name, err);
                    startup_executed = vec![false;config.groups.len()];
                    browser.restart().await?;
                },
                _ => {}
            }

            // perform startup steps if not already performed
            if !startup_executed[index] {
                process_steps(&group.startup, browser).await?;
                startup_executed[index] = true;
            }

            // perform main steps
            match process_steps(&group.steps, browser).await {
                Ok(_) => {
                    // one of the groups ran to success, stop
                    println!("Group [{}] success, stopping", group.name);
                    return Ok(());
                },
                // ignore errors on process steps, restart (happens automatically)
                Err(err) => {
                    match err  {
                        BrowserOutcome::NoSuchElement(_) => {
                            // silently ignore, expected error
                        },
                        unexpected @ _ => {
                            println!("Group [{}] unexpected error, restarting: {}", group.name, unexpected);
                            startup_executed = vec![false;config.groups.len()];
                            browser.restart().await?;
                        },
                    }
                },
            };
        }
    }
}

struct Sequences {
    pub inclusive: HashSet<String>,
    pub exclusive: HashSet<String>,
}

impl Sequences {
    pub fn new() -> Sequences {
        Sequences {
            inclusive: HashSet::new(),
            exclusive: HashSet::new(),
        }
    }
}

async fn process_steps(steps: &Vec<Step>, browser: &mut Browser) -> Result<(), BrowserOutcome> {
    let mut sequences = Sequences::new();

    for step in steps {
        process_step(step, browser, &mut sequences).await?;
    }
    // all steps done
    Ok(())
}

// per step
// - OK means success => do normal checks
// - cmd error means action failed => do normal checks
// - timeout error means browser issue => log & restart
async fn process_step(step: &Step, browser: &mut Browser, sequences: &mut Sequences) -> Result<(), BrowserOutcome> {
    match step {
        Step::Find{name, selector, action, optional_group, wait_max, delay, logging} => {
            // if not in optional group, execute with error
            // if in non-failed optional group and fails, add to failed options
            // if in failed optional group, skip
            match sequences.inclusive.get(optional_group) {
                None => {
                    // println!("Step [{}] starting", name);
                    match process_find(browser,selector, action, wait_max, delay).await {
                        Ok(_) => {
                            log(format!("Step [{}] success", name),logging);
                        },
                        Err(err) => {
                            match err  {
                                BrowserOutcome::NoSuchElement(_) => {
                                    if optional_group != "" {
                                        log(format!("Step [{}] option [{}] failed: {}", name, optional_group, err),logging);
                                        sequences.inclusive.insert(optional_group.to_string());
                                        return Ok(());
                                    } else {
                                        log(format!("Step [{}] failed, restarting: {}", name, err),logging);
                                        return Err(err);
                                    }
                                },
                                unexpected @ _ => { return Err(unexpected) },
                            }
                        }
                    };
                },
                Some(_) => {
                    log(format!("Optional step [{}] skipped", name),logging);
                }
            }
            Ok(())
        },
        Step::Navigate(dest) => {
            browser.goto(dest).await
        },
        Step::Refresh => {
            browser.refresh().await
        },
        Step::Log(message) => {
            println!("{}",message);
            Ok(())
        },
        Step::Wait(time) => {
            thread::sleep(time::Duration::from_millis(*time));
            Ok(())
        },
    }
}

fn log(message: String, log: &bool) {
    if *log {
        println!("{}",message);
    }
}