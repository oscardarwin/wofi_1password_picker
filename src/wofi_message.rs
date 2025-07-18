use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

pub fn wofi_message(title: &str, message: &str) -> Result<()> {
    let mut child = Command::new("wofi")
        .args(["--dmenu", "--prompt", title])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start wofi for message")?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .context("Failed to write to wofi stdin")?;
        writeln!(stdin, "{message}")?;
    }

    child.wait().context("Failed to wait for wofi message")?;
    Ok(())
}
