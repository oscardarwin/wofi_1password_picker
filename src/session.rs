use std::process::Command;

use anyhow::{Context, Result};

use crate::wofi::message;

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
    for line in env.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key.starts_with("OP_SESSION") {
                let session = value.to_string();
                println!("{:?}", session);
                return Ok(session);
            }
        }
    }

    message(
        "🔐 Not signed in to 1Password",
        "Please run `op signin` in a terminal or the 1Password GUI.\n\nThis launcher requires an active OP_SESSION variable.",
    )?;

    Err(anyhow::anyhow!(
        "No OP_SESSION_* variable found in systemd environment"
    ))
}
