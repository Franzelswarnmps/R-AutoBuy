use crate::config::*;
use fantoccini::{Client, Locator};
use std::time::{Instant};
use std::{thread, time};

pub async fn process_find(client: &mut Client, selector: &String, action: &FindAction, wait_max: &u64, delay: &u64) -> Result<(), fantoccini::error::CmdError> {
    let start_time = Instant::now();

    loop {
        if *delay > 0 {
            thread::sleep(time::Duration::from_millis(*delay));
        }
        match process_action(client, selector, action).await {
            Ok(_) => { return Ok(()) },
            Err(err) => { 
                if start_time.elapsed().as_millis() as u64 >= *wait_max {
                    return Result::Err(err);
                }
            },
        }
    }
}

pub async fn process_action(client: &mut Client, selector: &String, action: &FindAction) -> Result<(), fantoccini::error::CmdError> {

    match action {
        FindAction::Click => {
            client.find(Locator::Css(&selector)).await?.click().await?;
        }
        FindAction::Insert(value) => match client.form(Locator::Css("html")).await {
            Ok(mut val) => {
                val.set(Locator::Css(&selector), value).await?;
            },
            Err(err) => {
                return Err(err);
            }
        },
        FindAction::Compare(comparator) => {
            match comparator {
                Compare::Equal(value) => {
                    let found_value = &client.find(Locator::Css(&selector)).await?.text().await?;
                    match found_value == value {
                        true => {
                            return Ok(());
                        },
                        false => {
                            let failed_comparison = format!("{} != {}", found_value, value);
                            return Err(fantoccini::error::CmdError::InvalidArgument("Comparison failed".to_string(),failed_comparison));
                        }
                    }
                }
            }
        }
        FindAction::None => {
            client.find(Locator::Css(&selector)).await?;
        }
    }

    Ok(())
}