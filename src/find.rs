use crate::config::*;
use fantoccini::{Client, Locator};

pub async fn process_find(client: &mut Client, selector: &String, action: &FindAction) -> Result<(), fantoccini::error::CmdError> {

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