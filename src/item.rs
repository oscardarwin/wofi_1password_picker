use std::process::Command;

use anyhow::{Context, Result, anyhow};
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
    pub label: Option<String>,
    pub primary: Option<bool>,
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
    pub totp: Option<String>,
    pub reference: Option<String>,
    pub password_details: Option<PasswordDetails>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordDetails {
    pub strength: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DisplayedField {
    pub label: String,
    pub value: String,
}

fn get_item_from_1password(session: &Session, item_desc: &ItemDescription) -> Result<Item> {
    println!("Grabbing item with id {}", item_desc.id);
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
    let Item {
        fields: Some(fields),
        ..
    } = get_item_from_1password(session, item_description)?
    else {
        return Err(anyhow!("Item has no fields"));
    };
    let mut displayed_fields = get_displayed_fields(fields);
    sort_fields_by_priority(&mut displayed_fields);
    let labels_and_values = get_field_table(&displayed_fields)?;

    let index = wofi::select("📋 Copy field", labels_and_values)?;

    let selected_item = displayed_fields
        .into_iter()
        .nth(index)
        .map(|displayed_field| displayed_field.value.clone())
        .context("No value found for selected field")?;

    Ok(selected_item)
}

fn get_field_table(fields: &Vec<DisplayedField>) -> Result<Vec<[String; 2]>> {
    let labels_and_values: Vec<[String; 2]> = fields
        .iter()
        .map(|displayed_field| {
            let label = displayed_field.label.clone();
            let display_value = match displayed_field.label.as_str() {
                "one-time password" => "******".to_string(),
                "password" => "********".to_string(),
                _ => displayed_field.value.clone(),
            };
            [label, display_value]
        })
        .collect();

    if labels_and_values.is_empty() {
        return Err(anyhow::anyhow!("No fields with values found"));
    }

    Ok(labels_and_values)
}

fn get_displayed_fields(fields: Vec<Field>) -> Vec<DisplayedField> {
    fields
        .into_iter()
        .filter_map(|field| match field {
            Field {
                label,
                totp: Some(totp),
                ..
            } => Some(DisplayedField { label, value: totp }),
            Field {
                label,
                value: Some(value),
                ..
            } => Some(DisplayedField { label, value }),
            _ => None,
        })
        .collect()
}

pub fn sort_fields_by_priority(fields: &mut Vec<DisplayedField>) {
    fn priority(label: &str) -> usize {
        match label {
            "password" => 0,
            "one-time password" => 1,
            _ => 2,
        }
    }

    fields.sort_by_key(|f| priority(&f.label));
}
