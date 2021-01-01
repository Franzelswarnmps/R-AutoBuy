mod config;
mod step;
mod browser;
mod group;

use config::*;
use group::*;
use browser::*;
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let config = load_config("sites.toml")?;

    let mut browser = Browser::new(
        config.groups.len(),Duration::from_millis(config.timeout),
        &config.profile,
        &config.screenshot_path,
        config.marionette_port,
    ).await?;

    loop {
        for (index,group) in config.groups.iter().enumerate() {

            match browser.switch_tab(index).await {
                Err(err) => {
                    println!("Group [{}] tab switch error {}, restarting", group.name, err);
                    browser.restart().await?;
                },
                _ => {}
            }

            // decide whether to continue looping over groups
            match process_group(group, &mut browser).await {
                Ok(_) => {
                    println!("Ended OK");
                    browser.close().await?;
                    return Ok(());
                },
                Err(err) => {
                    match err  {
                        unexpected @ BrowserOutcome::Timeout(_) 
                        | unexpected @ BrowserOutcome::Unexpected(_) 
                        | unexpected @ BrowserOutcome::ClientLost => {
                            println!("Group [{}] unexpected error, restarting: {}", group.name, unexpected);
                            browser.restart().await?;
                        },
                        _ => {
                            // silently continue looping, expected error
                        },
                    }
                },
            };
        }
    }
}