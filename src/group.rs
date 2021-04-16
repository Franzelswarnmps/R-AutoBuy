use crate::config::*;
use crate::browser::*;
use crate::step::*;
use std::collections::{HashSet};

// handle if_not_cond, if_cond, optional, and logging inside sites.toml
pub async fn process_group(group: &Group, browser: &mut Browser) -> Result<(), BrowserOutcome> {

    let mut success: HashSet<String> = HashSet::new();
    let mut failed: HashSet<String> = HashSet::new();

    for step in &group.steps {
        if (step.if_cond == "" && step.if_not_cond == "")
        || (step.if_cond != "" && success.contains(&step.if_cond))
        || (step.if_not_cond != "" && failed.contains(&step.if_not_cond)) {

            match process_step(step, browser).await {
                Err(err) => {
                    log(format!("Step [{}:{}] failed", group.name,step.name ),&step.logging);
                    if !step.optional {
                        if step.logging {
                            match browser.screenshot().await {
                                Err(err) => {
                                    log(format!("Step [{}:{}] {}", group.name,step.name, err),&step.logging);
                                },
                                _ => {}
                            }
                        }
                        return Err(err);
                    }
                    if step.name != "" {
                        failed.insert(step.name.clone());
                    }
                },
                Ok(_) => {
                    log(format!("Step [{}:{}] success", group.name,step.name ),&step.logging);
                    if step.name != "" {
                        success.insert(step.name.clone());
                    }
                },
            }
        }
    }
    
    Ok(())
}

pub fn log(message: String, log: &bool) {
    if *log {
        println!("{}",message);
    }
}