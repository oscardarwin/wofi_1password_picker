use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn wofi_select_table<const N: usize>(
    title: &str,
    prompt: &str,
    entries: Vec<[String; N]>,
) -> Result<usize> {
    if entries.is_empty() {
        return Err(anyhow::anyhow!("No entries to display"));
    }

    // 1. Compute max width of each column
    let mut col_widths = [0usize; N];
    for row in &entries {
        for (i, col) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(col.len());
        }
    }

    // 2. Format entries with padding
    let formatted_lines: Vec<String> = entries
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let padded_cols: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, col)| format!("{:width$}", col, width = col_widths[i]))
                .collect();
            format!("{} ::index:{}", padded_cols.join("  |  "), index)
        })
        .collect();

    // 3. Launch wofi
    let mut child = Command::new("wofi")
        .args(["--dmenu", "--prompt", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to launch wofi")?;

    {
        let stdin = child.stdin.as_mut().context("Failed to open wofi stdin")?;
        for line in &formatted_lines {
            writeln!(stdin, "{}", line)?;
        }
    }

    let output = child
        .wait_with_output()
        .context("Failed to read wofi output")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("wofi exited non-successfully"));
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        return Err(anyhow::anyhow!("No selection made"));
    }

    let index = selected
        .rsplitn(2, "::index:")
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .context("Failed to parse selected index")?;

    Ok(index)
}
