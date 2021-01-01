use std::process::{Command, Stdio};
use std::error::Error;

use tokio::time::timeout;
use std::time::Duration;
use fantoccini::{Client, Locator, Element};

#[derive(Debug)]
pub enum BrowserOutcome {
    // normal errors, continue
    NoSuchElement(fantoccini::error::CmdError),
    EarlyEnd,
    Screenshot(String),
    MatchURLFail(String),

    // try restarting
    Timeout(tokio::time::Elapsed),
    Unexpected(fantoccini::error::CmdError),
    ClientLost,
}

impl Error for BrowserOutcome {}

impl std::fmt::Display for BrowserOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserOutcome::NoSuchElement(err) => {write!(f, "NoSuchElement error: ({})", err)}
            BrowserOutcome::Timeout(err) => {write!(f, "Timeout error: ({})", err)}
            BrowserOutcome::Unexpected(err) => {write!(f, "Unexpected error: ({})", err)},
            BrowserOutcome::EarlyEnd => {write!(f, "Manual end by step")},
            BrowserOutcome::ClientLost => {write!(f, "Client lost")},
            BrowserOutcome::Screenshot(name) => {write!(f, "Failed to take screenshot: ({})",name)},
            BrowserOutcome::MatchURLFail(name) => {write!(f, "Failed to match url: ({})",name)},
        }
    }
}

#[derive(Debug)]
struct TabDoesNotExist;

impl Error for TabDoesNotExist {}

impl std::fmt::Display for TabDoesNotExist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Browser tab does not exist")
    }
}

pub struct Browser {
    client: Option<Client>,
    pub timeout: Duration,
    pub profile: String,
    pub screenshot_path: String,
    pub marionette_port: u64,
    pub timestamp: u64,
    pub screenshot_counter: u64,
}

impl Browser {
    pub async fn new(tabs: usize, 
        timeout: Duration, 
        profile: &String,
        screenshot_path: &String,
        marionette_port: u64) -> Result<Browser, Box<dyn Error>> {
            
        Browser::close_driver().await.ok();
        Browser::force_close_firefox().await.ok();

        let mut browser = Browser {
            //client: Client::new("http://localhost:4444").await?,
            client: Some(Browser::new_client(profile,marionette_port).await?),
            timeout: timeout,
            profile: profile.clone(),
            screenshot_path: screenshot_path.clone(),
            marionette_port: marionette_port.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            screenshot_counter: 0,
        };

        for _ in 1..tabs {
            Browser::handle_result(browser.get_client().await?.new_window(true),timeout).await?;
        }

        Ok(browser)
    }

    pub async fn restart(&mut self) -> Result<(), Box<dyn Error>> {        
        self.close().await?;
        println!("====== BROWSER CLOSED ======");
        self.client = Some(Browser::new_client(&self.profile, self.marionette_port).await?);
        println!("====== NEW BROWSER MADE ======");

        //std::mem::replace(&mut *self.client, Browser::new_client(&self.profile, self.marionette_port).await?);
        Ok(())
    }

    async fn new_client(profile: &String, marionette_port: u64) -> Result<Client, Box<dyn Error>> {
        Command::new(".\\geckodriver.exe").
        args(&["--marionette-port", marionette_port.to_string().as_str()])
        .stdout(Stdio::null()).spawn()?;

        let args = serde_json::json![{
            "args": ["--profile", serde_json::value::Value::String(profile.clone())],
            // "prefs": {
            //     "marionette.port": serde_json::value::Value::Number(marionette_port.into())
            // }
        }];
        let mut capabilities = webdriver::capabilities::Capabilities::new();
        capabilities.insert("moz:firefoxOptions".to_string(), args);

        Ok(Client::with_capabilities("http://localhost:4444",capabilities).await?)
    }

    pub async fn switch_tab(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        let timeout = self.timeout;
        let handle = Browser::handle_result(self.get_client().await?.windows(), timeout).await?.get(index).ok_or(TabDoesNotExist)?.clone();
        Browser::handle_result(self.get_client().await?.switch_to_window(handle), timeout).await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn Error>> {
        Browser::force_close_firefox().await?;

        println!("====== TRYING TO CLOSE ======");

        self.get_client().await?.close().await?;

        println!("====== CLIENT CLOSED ======");

        Browser::close_driver().await?;

        println!("====== DRIVER CLOSED ======");

        Ok(())
    }

    async fn close_driver() -> Result<(), Box<dyn Error>> {
        Command::new("taskkill")
        .args(&["/f", "/im", "geckodriver.exe"])
        .output()?;
        Ok(())
    }

    async fn force_close_firefox() -> Result<(), Box<dyn Error>> {
        Command::new("taskkill")
        .args(&["/f", "/im", "Firefox.exe"])
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
        let timeout = self.timeout;
        Browser::handle_result(self.get_client().await?.find(Locator::Css(selector)),timeout).await
    }

    pub async fn click(&mut self, selector: &String) -> Result<(), BrowserOutcome>  {
        match Browser::handle_result(self.find(selector).await?.click(), self.timeout).await {
            Ok(_) => {Ok(())},
            Err(err) => {Err(err)}
        }
    }

    pub async fn insert(&mut self, selector: &String, value: &String) -> Result<(), BrowserOutcome>  {
        let timeout = self.timeout;

        match Browser::handle_result(self.get_client().await?.form(Locator::Css("html")),timeout).await {
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
        let timeout = self.timeout;
        match Browser::handle_result(self.get_client().await?.goto(dest),timeout).await {
            Ok(_) => {Ok(())},
            Err(err) => {Err(err)}
        }
    }

    pub async fn refresh(&mut self) -> Result<(), BrowserOutcome>  {
        let timeout = self.timeout;
        match Browser::handle_result(self.get_client().await?.refresh(),timeout).await {
            Ok(_) => {Ok(())},
            Err(err) => {Err(err)}
        }
    }

    pub async fn screenshot(&mut self) -> Result<(), BrowserOutcome> {
        let timeout = self.timeout;

        let full_path = format!("{}{}-{}.png", self.screenshot_path, self.timestamp, self.screenshot_counter);
        let (width,height) = Browser::handle_result(self.get_client().await?.get_window_size(),timeout).await?;
        let pixels = Browser::handle_result(self.get_client().await?.screenshot(),timeout).await?;
        let image = match image::RgbImage::from_raw(width as u32, height as u32, pixels) {
            Some(val) => val,
            None => {
                return Err(BrowserOutcome::Screenshot(
                    format!("{}, {}",full_path.clone(),"raw conversion failed")
                ));
            },
        };
        match image.save(full_path.clone()) {
            Err(err) => {
                return Err(BrowserOutcome::Screenshot(
                    format!("{}, {}",full_path.clone(),err))
                );
            },
            _ => {
                self.screenshot_counter += 1;
            }
        }
        Ok(())
    }

    pub async fn current_url(&mut self) -> Result<String, BrowserOutcome> {
        let timeout = self.timeout;

        Ok(Browser::handle_result(
            self.get_client().await?.current_url(),timeout
        ).await?.to_string())
    }

    // builder method used due to underlying calls
    pub async fn top_window(&mut self) -> Result<(), BrowserOutcome> {
        let client = match std::mem::take(&mut self.client) {
            Some(val) => {val},
            None => { return Err(BrowserOutcome::ClientLost) }
        };

        self.client = Some(Browser::handle_result(client.enter_parent_frame(), self.timeout).await?);

        Ok(())
    }

    // builder method used due to underlying calls
    pub async fn switch_frame(&mut self,element: Element) -> Result<(), BrowserOutcome> {
        Browser::handle_result(element.enter_frame(), self.timeout).await?;
        Ok(())
    }

    async fn get_client(&mut self) -> Result<&mut Client, BrowserOutcome> {
        match &mut self.client {
            Some(val) => {Ok(val)},
            None => { Err(BrowserOutcome::ClientLost) }
        }
    }
}