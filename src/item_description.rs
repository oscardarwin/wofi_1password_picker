use anyhow::{Context, Result};
use std::process::Command;

use serde::Deserialize;

use crate::{session::Session, wofi_message::wofi_message, wofi_select::wofi_select_table};

#[derive(Debug, Deserialize, Clone)]
pub struct ItemDescription {
    pub id: String,
    pub title: String,
    pub version: u32,
    pub vault: Vault,
    pub category: String,
    pub last_edited_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub additional_information: Option<String>,
    pub urls: Option<Vec<Url>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Vault {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Url {
    pub primary: Option<bool>,
    pub href: String,
}

fn get_item_descriptions(session: &Session) -> Result<Vec<ItemDescription>> {
    let item_list = Command::new("op")
        .args(["item", "list", "--format", "json", "--session", session])
        .output()
        .context("Failed to list 1Password items")?;
    let stdout = String::from_utf8(item_list.stdout)?;
    if stdout.trim().is_empty() {
        let message = "No 1Password items found.\n\nCheck your vault or sign-in status.";
        wofi_message("🔍 No Items", &message)?;
        return Err(anyhow::anyhow!(message));
    }

    let items: Vec<ItemDescription> = serde_json::from_str(&stdout)?;
    Ok(items)
}

pub fn select_item_description(session: &Session) -> Result<ItemDescription> {
    let item_descriptions = get_item_descriptions(session)?;
    let rows: Vec<[String; 3]> = item_descriptions
        .iter()
        .map(|item| {
            let title = item.title.clone();
            let info = item.additional_information.clone().unwrap_or_default();
            let url = item
                .urls
                .as_ref()
                .and_then(|urls| urls.first())
                .map(|u| u.href.clone())
                .unwrap_or_default();
            [title, info, url]
        })
        .collect();

    let index = wofi_select_table("🔐 1Password", "Select login", rows)?;
    item_descriptions
        .get(index)
        .cloned()
        .context("Selected index out of bounds")
}
