use anyhow::Result;
use std::io::{self, Write};

pub fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush().ok();
    let pwd = rpassword::read_password()?;
    Ok(pwd)
}

pub fn confirm_overwrite(path: &str) -> Result<bool> {
    println!("⚠ File '{}' already exists. Overwrite? [y/N]", path);
    print!("> ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let ans = input.trim().to_lowercase();
    Ok(ans == "y" || ans == "yes")
}
