use crate::{
    core::crypto::Meta,
    utils::{git, prompt, tool},
};
use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

mod crypto;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvLockConfig {
    env_file: String,
    encrypted_file: String,
    meta_file: String,
}

pub fn cmd_init(env_path: &str, enc_path: &str, meta_path: &str) -> Result<()> {
    let config_path = ".envlock/config.json";

    if !Path::new(config_path).exists() {
        tool::ensure_folders_of_path(config_path)
            .with_context(|| format!("Failed to create directory {}", config_path).red())?;

        let default_config = EnvLockConfig {
            env_file: String::from(env_path),
            encrypted_file: String::from(enc_path),
            meta_file: String::from(meta_path),
        };
        let config_json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, config_json)
            .with_context(|| format!("Failed to write {}", config_path))?;
        println!("{}", "✓ EnvLock initialized successfully.".green());
    } else {
        println!("{}", "✓ EnvLock already initialized.".yellow());
    }

    Ok(())
}

pub fn cmd_lock(env_path: &str, enc_path: &str, meta_path: &str, force: bool) -> Result<()> {
    println!("🔐 Encrypting {} …", env_path);

    if !Path::new(env_path).exists() {
        return Err(anyhow!(format!("Env file '{}' not found", env_path).red()));
    }

    if Path::new(enc_path).exists() && !force {
        if !prompt::confirm_overwrite(enc_path)? {
            println!("Aborted.");
            return Ok(());
        }
    }

    let plaintext =
        fs::read_to_string(env_path).with_context(|| format!("Failed to read {}", env_path))?;

    let password = prompt::prompt_password("Enter password: ")?;
    let confirm = prompt::prompt_password("Confirm password: ")?;
    if password != confirm {
        return Err(anyhow!("Passwords do not match".red()));
    }

    let (ciphertext, meta) = crypto::encrypt_env(&plaintext, &password)?;
    tool::ensure_folders_of_path(enc_path)
        .with_context(|| format!("Failed to create directory {}", enc_path).red())?;
    fs::write(enc_path, ciphertext).with_context(|| format!("Failed to write {}", enc_path))?;

    let meta_json = serde_json::to_string_pretty(&meta)?;
    tool::ensure_folders_of_path(meta_path)
        .with_context(|| format!("Failed to create directory {}", meta_path).red())?;
    fs::write(meta_path, meta_json)
        .with_context(|| format!("Failed to write {}", meta_path).red())?;

    println!(
        "{}",
        format!("✓ Encrypted file saved: {}", enc_path).green()
    );

    println!("{}", format!("✓ Metadata saved: {}", meta_path).green());
    Ok(())
}

pub fn cmd_unlock(env_path: &str, enc_path: &str, meta_path: &str, force: bool) -> Result<()> {
    println!("🔓 Decrypting {} …", enc_path);

    if !Path::new(enc_path).exists() {
        return Err(anyhow!(
            format!("Encrypted file '{}' not found", enc_path).red()
        ));
    }
    if !Path::new(meta_path).exists() {
        return Err(anyhow!(
            format!("Metadata file '{}' not found", meta_path).red()
        ));
    }

    if Path::new(env_path).exists() && !force {
        if !prompt::confirm_overwrite(env_path)? {
            println!("Aborted.");
            return Ok(());
        }
    }

    let ciphertext =
        fs::read(enc_path).with_context(|| format!("Failed to read {}", enc_path).red())?;
    let meta_json = fs::read_to_string(meta_path)
        .with_context(|| format!("Failed to read {}", meta_path).red())?;
    let meta: Meta = serde_json::from_str(&meta_json).context("Invalid metadata JSON")?;

    let password = prompt::prompt_password("Enter password: ")?;

    let plaintext =
        crypto::decrypt_env(&ciphertext, &password, &meta).map_err(|e| match e
            .downcast_ref::<crypto::EnvLockError>()
        {
            Some(crypto::EnvLockError::DecryptionError) => {
                anyhow!("Failed to decrypt: wrong password or corrupted file")
            }
            None => e,
        })?;

    fs::write(env_path, plaintext)
        .with_context(|| format!("Failed to write {}", env_path).red())?;

    println!(
        "{}",
        format!("✓ Decrypted env written to {}", env_path).green()
    );

    Ok(())
}

pub fn cmd_diff(env_path: &str, enc_path: &str, meta_path: &str) -> Result<()> {
    println!("🔍 Diffing {} and {} …", env_path, enc_path);

    if !Path::new(env_path).exists() {
        return Err(anyhow!(format!("Env file '{}' not found", env_path).red()));
    }
    if !Path::new(enc_path).exists() {
        return Err(anyhow!(
            format!("Encrypted file '{}' not found", enc_path).red()
        ));
    }
    if !Path::new(meta_path).exists() {
        return Err(anyhow!(
            format!("Metadata file '{}' not found", meta_path).red()
        ));
    }

    let env_plain = fs::read_to_string(env_path)
        .with_context(|| format!("Failed to read {}", env_path).red())?;

    let ciphertext =
        fs::read(enc_path).with_context(|| format!("Failed to read {}", enc_path).red())?;
    let meta_json = fs::read_to_string(meta_path)
        .with_context(|| format!("Failed to read {}", meta_path).red())?;
    let meta: Meta = serde_json::from_str(&meta_json).context("Invalid metadata JSON")?;

    let password = prompt::prompt_password("Enter password: ")?;
    let decrypted =
        crypto::decrypt_env(&ciphertext, &password, &meta).map_err(|e| match e
            .downcast_ref::<crypto::EnvLockError>()
        {
            Some(crypto::EnvLockError::DecryptionError) => {
                anyhow!("Failed to decrypt: wrong password or corrupted file".red())
            }
            None => e,
        })?;

    let current_env = tool::parse_env(&env_plain);
    let decrypted_env = tool::parse_env(&decrypted);

    tool::print_diff(&current_env, &decrypted_env);

    Ok(())
}

pub fn cmd_sync(env_path: &str, enc_path: &str, meta_path: &str, message: &str) -> Result<()> {
    println!("🔄 Syncing env with git …");

    // 1) lock (encrypt) if .enc not exists
    cmd_lock(env_path, enc_path, meta_path, true)?;

    // 2) git add
    git::run_git(&["add", enc_path, meta_path])?;

    // 3) git commit
    git::run_git(&["commit", "-m", message])?;

    // 4) git push
    git::run_git(&["push"])?;

    println!("{}", "✓ Sync completed.".green());
    Ok(())
}
