use crate::config::*;
use crate::browser::*;
use std::time::{Instant};
use std::{thread, time};

// process a single step in sites.toml
// handle waiting/retry here
// return actual result
pub async fn process_step(step: &Step, browser: &mut Browser) -> Result<(), BrowserOutcome> {
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

// process a step's action in sites.toml
pub async fn process_action(step: &Step, browser: &mut Browser) -> Result<(), BrowserOutcome> {

    match &step.action {
        StepAction::Navigate{url, anti_cache} => {
            let mut final_url = url.clone();
            if *anti_cache {
                final_url = format!("{}?{}",final_url,rand::random::<u64>());
            }
            browser.goto(&final_url).await?
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
                },
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
        StepAction::Special(action) => {
            match action {
                SpecialAction::SolveAmazonReCaptcha => {
                    let image_selector = "form[action='/errors/validateCaptcha'] img".to_string();
                    let image_attr = "src".to_string();

                    let image_url;
                    match browser.find_attribute(&image_selector,&image_attr).await? {
                        Some(val) => {image_url = val},
                        None => {
                            return Err(BrowserOutcome::ReCaptchaIssue("Missing src attribute on ReCaptcha img tag".to_string()));
                        }
                    }

                    let answer = pyo3::Python::with_gil(|py| -> Result<String,pyo3::PyErr> {
                        use pyo3::types::*;
                        let locals = [("captcha", py.import("amazoncaptcha")?)].into_py_dict(py);
                        let code = format!("captcha.AmazonCaptcha.fromlink('{}').solve()",image_url);
                        let result: String = py.eval(code.as_str(), None, Some(&locals))?.extract()?;
                        Ok(result)
                    }).map_err(|err| {
                        pyo3::Python::with_gil(|py| err.print_and_set_sys_last_vars(py));
                        BrowserOutcome::ReCaptchaIssue("Problem with the Python invocation".to_string())
                    })?;

                    let insert_selector ="#captchacharacters".to_string();
                    browser.insert(&insert_selector,&answer).await?;

                    let submit_selector = "form[action='/errors/validateCaptcha'] button[type='submit']".to_string();
                    browser.click(&submit_selector).await?
                },
            }
        },
    }

    Ok(())
}