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
async fn main() {

    let mut config = match load_config("sites.toml") {
        Ok(val) => val,
        Err(err) => {
            println!("problem with config: {}",err);
            return;
        }
    };

    let secrets = match load_secrets("secrets.toml") {
        Ok(val) => val,
        Err(err) => {
            println!("problem with secrets: {}",err);
            return;
        }
    };

    populate_secrets(&mut config, &secrets);

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
    // let mut start_time = Instant::now();
    let mut browser = Browser::new(Duration::from_millis(config.timeout)).await?;

    loop {
        // if config.restart != 0 && start_time.elapsed().as_millis() as u64 >= config.restart {
        //     start_time = Instant::now();
        //     close_browser(&mut client).await?;
        //     client = open_browser().await?;
        // }

        for group in &config.parallel_groups {
            match process_steps(group, &mut browser).await {
                Ok(_) => {
                    // one of the groups ran to success, stop
                    println!("Group [{}] success, stopping", group.name);
                    return Ok(());
                },
                // ignore errors on process steps, restart (happens automatically)
                Err(_) => { },
            };
        }

        // client health check 
        // match browser.client.refresh().await {
        //     Ok(_) => {},
        //     Err(err) => {
        //         close_browser(&mut client).await?;
        //         return Result::Err(Box::new(err));
        //     }
        // };
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

async fn process_steps(group: &ParallelGroup, browser: &mut Browser) -> Result<(), BrowserOutcome> {
    let mut sequences = Sequences::new();

    for step in &group.steps {
        match process_step(step, browser, &mut sequences).await?
    }
    // all steps done
    Ok(())
}

// per step
// - OK means success => do normal checks
// - cmd error means action failed => do normal checks
// - timeout error means browser issue => log & restart
async fn process_step(step: &Step, browser: &mut Browser, sequences: &mut Sequences) -> Result<(), BrowserOutcome> {
        //println!("starting step {}", step.name);
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
                                    nested_err @ BrowserOutcome::Unexpected(_) => {
                                        return Err(nested_err);
                                    },
                                    nested_err @ BrowserOutcome::Timeout(_) => {
                                        return Err(nested_err);
                                    },
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
                                }
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