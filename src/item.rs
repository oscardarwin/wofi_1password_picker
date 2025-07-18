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
    let fields = item.fields.as_ref().context("Item has no fields")?;
    let mut displayable_fields = get_displayable_fields(&fields);
    sort_fields_by_priority(&mut displayable_fields);
    let labels_and_values = get_field_table(&displayable_fields)?;

    let index = wofi::select("📋 Copy field", labels_and_values)?;

    let selected_item = displayable_fields
        .iter()
        .nth(index)
        .and_then(|f| f.value.clone())
        .context("No value found for selected field")?;

    Ok(selected_item)
}

fn get_field_table(fields: &Vec<&Field>) -> Result<Vec<[String; 2]>> {
    let labels_and_values: Vec<[String; 2]> = fields
        .iter()
        .filter_map(|label_and_value| {
            let label = label_and_value.label.clone();
            let display_value = match label_and_value.purpose.as_deref() {
                Some("PASSWORD") => Some("********".to_string()),
                _ => label_and_value.value.clone(),
            }?;
            Some([label, display_value])
        })
        .collect();

    if labels_and_values.is_empty() {
        return Err(anyhow::anyhow!("No fields with values found"));
    }

    Ok(labels_and_values)
}

fn get_displayable_fields(fields: &Vec<Field>) -> Vec<&Field> {
    fields.iter().filter(|f| f.value.is_some()).collect()
}

pub fn sort_fields_by_priority(fields: &mut Vec<&Field>) {
    fn priority(purpose: Option<&str>) -> usize {
        match purpose {
            Some("PASSWORD") => 0,
            Some("OTP") | Some("ONE_TIME_PASSWORD") => 1,
            _ => 2,
        }
    }

    fields.sort_by_key(|f| priority(f.purpose.as_deref()));
}
