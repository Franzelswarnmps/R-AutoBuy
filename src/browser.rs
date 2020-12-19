use std::process::{Command, Stdio};
use fantoccini::Client;
use std::error::Error;

pub async fn open_browser() -> Result<Client, Box<dyn Error>> {
    Command::new(".\\geckodriver.exe").stdout(Stdio::null()).spawn()?;
    Ok(Client::new("http://localhost:4444").await?)
}

pub async fn close_browser(client: &mut Client) -> Result<(), Box<dyn Error>> {
    client.close().await?;

    Command::new("taskkill")
    .args(&["/f", "/im", "geckodriver.exe"])
    .output()?;

    Ok(())
}