use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{item_description::ItemDescription, session::Session, wofi};

#[derive(Debug, Deserialize)]
pub struct Item {
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
    pub fields: Option<Vec<Field>>,
}

#[derive(Debug, Deserialize)]
pub struct Vault {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Url {
    pub primary: bool,
    pub href: String,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub id: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub purpose: Option<String>,
    pub label: String,
    pub value: Option<String>,
    pub reference: Option<String>,
    pub password_details: Option<PasswordDetails>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordDetails {
    pub strength: Option<String>,
}

fn get_item_from_1password(session: &Session, item_desc: &ItemDescription) -> Result<Item> {
    let output = Command::new("op")
        .args([
            "item",
            "get",
            &item_desc.id,
            "--format",
            "json",
            "--session",
            session,
        ])
        .output()
        .context("Failed to get full item")?;

    serde_json::from_slice(&output.stdout).context("Failed to parse full item JSON")
}

pub fn select_field_to_copy(
    session: &Session,
    item_description: &ItemDescription,
) -> Result<String> {
    let item = get_item_from_1password(session, item_description)?;
    let (field_labels, field_values) = get_displayable_fields(&item)?;

    let rows: Vec<[String; 2]> = field_labels
        .into_iter()
        .zip(field_values)
        .map(|(label, value)| [label, value])
        .collect();

    let index = wofi::select("📋 Copy field", rows)?;
    Ok(item
        .fields
        .unwrap_or_default()
        .into_iter()
        .filter(|f| f.value.is_some())
        .nth(index)
        .and_then(|f| f.value)
        .context("No value found for selected field")?)
}

fn get_displayable_fields(item: &Item) -> Result<(Vec<String>, Vec<String>)> {
    let fields = item.fields.as_ref().context("Item has no fields")?;

    let labels_and_values: Vec<_> = fields
        .iter()
        .filter_map(|f| f.value.as_ref().map(|v| (f.label.clone(), v.clone())))
        .collect();

    if labels_and_values.is_empty() {
        return Err(anyhow::anyhow!("No fields with values found"));
    }

    let (labels, values): (Vec<_>, Vec<_>) = labels_and_values.into_iter().unzip();
    Ok((labels, values))
}
