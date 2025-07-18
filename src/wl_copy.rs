use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};

pub fn to_clipboard(field: String) -> Result<()> {
    let mut wl_copy = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to launch wl-copy")?;

    wl_copy
        .stdin
        .as_mut()
        .context("Failed to open wl-copy stdin")?
        .write_all(field.as_bytes())?;

    // 9. Optionally wait for clipboard copy to finish
    wl_copy.wait()?;

    Ok(())
}
