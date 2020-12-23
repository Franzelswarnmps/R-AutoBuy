use crate::config::*;
use crate::browser::*;
use std::time::{Instant};
use std::{thread, time};

pub async fn process_find(browser: &mut Browser, selector: &String, action: &FindAction, wait_max: &u64, delay: &u64) -> Result<(), BrowserOutcome> {
    let start_time = Instant::now();

    loop {
        if *delay > 0 {
            thread::sleep(time::Duration::from_millis(*delay));
        }
        match process_action(browser, selector, action).await {
            Ok(_) => { return Ok(()) },
            Err(err) => { 
                if start_time.elapsed().as_millis() as u64 >= *wait_max {
                    return Result::Err(err);
                }
            },
        }
    }
}

pub async fn process_action(browser: &mut Browser, selector: &String, action: &FindAction) -> Result<(), BrowserOutcome> {

    match action {
        FindAction::Click => {
            browser.click(selector).await?
        }
        FindAction::Insert(value) => {
            browser.insert(selector, value).await?
        },
        FindAction::Compare(comparator) => {
            match comparator {
                Compare::Equal(value) => {
                    let found_value = browser.text(selector).await?;
                    match found_value == *value {
                        true => {
                            return Ok(());
                        },
                        false => {
                            let failed_comparison = format!("{} != {}", found_value, value);
                            return Err(
                                BrowserOutcome::Unexpected(
                                    fantoccini::error::CmdError::InvalidArgument("Comparison failed".to_string(),failed_comparison)
                                )
                            );
                        }
                    }
                }
            }
        }
        FindAction::None => {
            browser.find(selector).await?;
        }
    }

    Ok(())
}