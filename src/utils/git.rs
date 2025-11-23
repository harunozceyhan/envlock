use std::process::Command;

use anyhow::{Result, anyhow};

pub fn run_git(args: &[&str]) -> Result<()> {
    let output = Command::new("git").args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git {:?} failed: {}", args, stderr.trim()));
    }
    Ok(())
}
