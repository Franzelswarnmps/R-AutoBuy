use crate::config::*;
use crate::browser::*;
use std::time::{Instant};
use std::{thread, time};

pub async fn process_step(step: &Step, browser: &mut Browser) -> Result<(), BrowserOutcome> {
    // handle waiting/retry here
    // return actual result
    let start_time = Instant::now();

    loop {
        if step.delay > 0 {
            thread::sleep(time::Duration::from_millis(step.delay));
        }
        match process_action(step, browser).await {
            Ok(_) => { return Ok(()) },
            Err(err) => { 
                if start_time.elapsed().as_millis() as u64 >= step.wait_max {
                    return Result::Err(err);
                }
            },
        }
    }
}

pub async fn process_action(step: &Step, browser: &mut Browser) -> Result<(), BrowserOutcome> {

    match &step.action {
        StepAction::Navigate(url) => {
            browser.goto(url).await?
        },
        StepAction::Wait(time) => {
            thread::sleep(time::Duration::from_millis(*time));
        },
        StepAction::Screenshot => {
            browser.screenshot().await?;
        },
        StepAction::MatchURL(url) => {
            if browser.current_url().await?.contains(url) {
                return Ok(());
            } else {
                return Err(BrowserOutcome::MatchURLFail(url.clone()));
            }
        },
        StepAction::Refresh => {
            browser.refresh().await?
        },
        StepAction::End => {
            return Err(BrowserOutcome::EarlyEnd);
        },
        StepAction::TopWindow => {
            browser.top_window().await?;
        },
        StepAction::Find{selector, action} => {
            match action {
                FindAction::Click => {
                    browser.click(selector).await?
                }
                FindAction::Insert(value) => {
                    browser.insert(selector, value).await?
                },
                FindAction::None => {
                    browser.find(selector).await?;
                },
                FindAction::SwitchFrame => {
                    let element = browser.find(selector).await?;
                    browser.switch_frame(element).await?;
                }
            }
        },
    }

    Ok(())
}