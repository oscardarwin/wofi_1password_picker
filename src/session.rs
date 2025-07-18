use std::process::Command;

use anyhow::{Context, Result};

use crate::wofi;

pub type Session = String;

pub fn get_op_session_from_systemd() -> Result<Session> {
    let output = Command::new("systemctl")
        .args(["--user", "show-environment"])
        .output()
        .context("Failed to query systemd user environment")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "`systemctl --user show-environment` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let env = String::from_utf8_lossy(&output.stdout);
    let session = env
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            if key == "OP_SESSION" {
                Some(value.to_string())
            } else {
                None
            }
        })
        .context("No OP_SESSION environment variable found in systemd user environment")?;

    let session_is_valid = is_session_valid(&session);

    if session_is_valid {
        Ok(session)
    } else {
        wofi::message(
            "🔐 Not signed in to 1Password",
            "Please run `op signin` in a terminal or the 1Password GUI.\nThis launcher requires an active OP_SESSION variable.",
        )?;

        Err(anyhow::anyhow!(
            "No OP_SESSION variable found in systemd environment"
        ))
    }
}

fn is_session_valid(session: &Session) -> bool {
    Command::new("op")
        .args(["item", "list", "--session", &session, "--format", "json"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
