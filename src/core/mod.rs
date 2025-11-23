use crate::{
    core::crypto::Meta,
    utils::{git, prompt, tool},
};
use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use std::{fs, path::Path};

mod crypto;

pub fn cmd_lock(env_path: &str, enc_path: &str, meta_path: &str, force: bool) -> Result<()> {
    println!("🔐 Encrypting {} …", env_path);

    if !Path::new(env_path).exists() {
        return Err(anyhow!("Env file '{}' not found", env_path));
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
        return Err(anyhow!("Passwords do not match"));
    }

    let (ciphertext, meta) = crypto::encrypt_env(&plaintext, &password)?;
    fs::write(enc_path, ciphertext).with_context(|| format!("Failed to write {}", enc_path))?;

    let meta_json = serde_json::to_string_pretty(&meta)?;
    fs::write(meta_path, meta_json).with_context(|| format!("Failed to write {}", meta_path))?;

    println!(
        "{} {}",
        "✓ Encrypted file saved:".yellow(),
        enc_path.yellow()
    );
    println!("{} {}", "✓ Metadata saved:".yellow(), meta_path.yellow());
    Ok(())
}

pub fn cmd_unlock(env_path: &str, enc_path: &str, meta_path: &str, force: bool) -> Result<()> {
    println!("🔓 Decrypting {} …", enc_path);

    if !Path::new(enc_path).exists() {
        return Err(anyhow!("Encrypted file '{}' not found", enc_path));
    }
    if !Path::new(meta_path).exists() {
        return Err(anyhow!("Metadata file '{}' not found", meta_path));
    }

    if Path::new(env_path).exists() && !force {
        if !prompt::confirm_overwrite(env_path)? {
            println!("Aborted.");
            return Ok(());
        }
    }

    let ciphertext = fs::read(enc_path).with_context(|| format!("Failed to read {}", enc_path))?;
    let meta_json =
        fs::read_to_string(meta_path).with_context(|| format!("Failed to read {}", meta_path))?;
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

    fs::write(env_path, plaintext).with_context(|| format!("Failed to write {}", env_path))?;

    println!("✓ Decrypted env written to {}", env_path);
    Ok(())
}

pub fn cmd_diff(env_path: &str, enc_path: &str, meta_path: &str) -> Result<()> {
    println!("🔍 Diffing {} and {} …", env_path, enc_path);

    if !Path::new(env_path).exists() {
        return Err(anyhow!("Env file '{}' not found", env_path));
    }
    if !Path::new(enc_path).exists() {
        return Err(anyhow!("Encrypted file '{}' not found", enc_path));
    }
    if !Path::new(meta_path).exists() {
        return Err(anyhow!("Metadata file '{}' not found", meta_path));
    }

    let env_plain =
        fs::read_to_string(env_path).with_context(|| format!("Failed to read {}", env_path))?;

    let ciphertext = fs::read(enc_path).with_context(|| format!("Failed to read {}", enc_path))?;
    let meta_json =
        fs::read_to_string(meta_path).with_context(|| format!("Failed to read {}", meta_path))?;
    let meta: Meta = serde_json::from_str(&meta_json).context("Invalid metadata JSON")?;

    let password = prompt::prompt_password("Enter password: ")?;
    let decrypted =
        crypto::decrypt_env(&ciphertext, &password, &meta).map_err(|e| match e
            .downcast_ref::<crypto::EnvLockError>()
        {
            Some(crypto::EnvLockError::DecryptionError) => {
                anyhow!("Failed to decrypt: wrong password or corrupted file")
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
