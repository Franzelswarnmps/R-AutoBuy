use std::process::{Command, Stdio};
use std::error::Error;

use tokio::time::timeout;
use std::time::Duration;
use fantoccini::{Client, Locator};

pub enum BrowserOutcome {
    NoSuchElement(fantoccini::error::CmdError),
    Timeout(tokio::time::Elapsed),
    Unexpected(fantoccini::error::CmdError),
}

impl std::fmt::Display for BrowserOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserOutcome::NoSuchElement(err) => {write!(f, "NoSuchElement error: ({})", err)}
            BrowserOutcome::Timeout(err) => {write!(f, "Timeout error: ({})", err)}
            BrowserOutcome::Unexpected(err) => {write!(f, "Unexpected error: ({})", err)}
        }
    }
}

pub struct Browser {
    pub client: Client,
    pub timeout: Duration,
}

impl Browser {
    pub async fn new(timeout: Duration) -> Result<Browser, Box<dyn Error>> {

        Browser::close_driver().await.ok();

        Command::new(".\\geckodriver.exe").stdout(Stdio::null()).spawn()?;

        Ok(Browser {
            client: Client::new("http://localhost:4444").await?,
            timeout: timeout,
        })
    }

    pub async fn restart(&mut self) -> Result<(), Box<dyn Error>> {
        self.close().await.ok();

        Command::new(".\\geckodriver.exe").stdout(Stdio::null()).spawn()?;
        self.client = Client::new("http://localhost:4444").await?;

        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), String> {
        let mut error_message = "".to_string();

        match self.client.close().await {
            Ok(_) => {},
            Err(err) => {
                error_message = format!("problem closing the client: {}", err);
            }
        }

        match Browser::close_driver().await {
            Ok(_) => {},
            Err(err) => {
                error_message = format!("{}, problem ending geckodriver: {}", error_message, err);
            }
        }
    
        if error_message.len() > 0 {
            return Result::Err(error_message.to_string())
        }
        Ok(())
    }

    async fn close_driver() -> Result<(), Box<dyn Error>> {
        Command::new("taskkill")
        .args(&["/f", "/im", "geckodriver.exe"])
        .output()?;
        Ok(())
    }

    async fn handle_result<T,X>(future: T, time: Duration)
    -> Result<X,BrowserOutcome>
    where T: std::future::Future<Output=std::result::Result<X, fantoccini::error::CmdError>> {
        match timeout(time,future).await {
            Ok(val) => {
                match val {
                    Ok(element) => {
                        Ok(element)
                    },
                    Err(err) => {
                        match err {
                            no_elem @ fantoccini::error::CmdError::NoSuchElement(_) => {
                                Err(BrowserOutcome::NoSuchElement(no_elem))
                            },
                            any @ _ => {
                                Err(BrowserOutcome::Unexpected(any))
                            }
                        }
                    }
                }
            },
            Err(err) => {
                Err(BrowserOutcome::Timeout(err))
            }
        }
    }

    pub async fn find(&mut self, selector: &String) -> Result<fantoccini::Element, BrowserOutcome> {
        Browser::handle_result(self.client.find(Locator::Css(selector)),self.timeout).await
    }

    pub async fn click(&mut self, selector: &String) -> Result<(), BrowserOutcome>  {
        match Browser::handle_result(self.find(selector).await?.click(), self.timeout).await {
            Ok(_) => {Ok(())},
            Err(err) => {Err(err)}
        }
    }

    pub async fn insert(&mut self, selector: &String, value: &String) -> Result<(), BrowserOutcome>  {
        match Browser::handle_result(self.client.form(Locator::Css("html")),self.timeout).await {
            Ok(mut val) => {
                match Browser::handle_result(val.set(Locator::Css(selector), value),self.timeout).await {         
                    Ok(_) => {Ok(())},
                    Err(err) => {Err(err)}
                }
            },
            Err(err) => {Err(err)}
        }
    }

    pub async fn goto(&mut self, dest: &String) -> Result<(), BrowserOutcome>  {
        match Browser::handle_result(self.client.goto(dest),self.timeout).await {
            Ok(_) => {Ok(())},
            Err(err) => {Err(err)}
        }
    }

    pub async fn refresh(&mut self) -> Result<(), BrowserOutcome>  {
        match Browser::handle_result(self.client.refresh(),self.timeout).await {
            Ok(_) => {Ok(())},
            Err(err) => {Err(err)}
        }
    }

    pub async fn text(&mut self, selector: &String) -> Result<String, BrowserOutcome>  {
        Browser::handle_result(self.find(selector).await?.text(), self.timeout).await
    }
}