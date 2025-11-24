use std::{fs, path::Path};

use crate::core;
use clap::{Args, Parser, Subcommand};

const DEFAULT_CONFIG_FILE: &str = "envlock.json";
const DEFAULT_ENV_FILE: &str = ".env";
const DEFAULT_ENC_FILE: &str = ".env.enc";
const DEFAULT_META_FILE: &str = ".env.meta.json";

/// CLI definition
#[derive(Parser, Debug)]
#[command(name = "envlock")]
#[command(about = "Secure .env encryption tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug, Clone)]
struct PathArgs {
    /// Path to .env file
    #[arg(short = 'e', long)]
    env: Option<String>,
    /// Path to .enc file
    #[arg(short = 'c', long)]
    enc: Option<String>,
    /// Path to .env.meta.json file
    #[arg(short = 'm', long)]
    meta: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Encrypt .env → .env.enc + .env.meta.json
    Lock {
        #[command(flatten)]
        paths: PathArgs,
        /// Overwrite existing .env.enc without asking
        #[arg(long)]
        force: bool,
    },

    /// Decrypt .env.enc → .env
    Unlock {
        #[command(flatten)]
        paths: PathArgs,
        /// Overwrite existing .env without asking
        #[arg(long)]
        force: bool,
    },

    /// Diff between current .env and decrypted .env.enc
    Diff {
        #[command(flatten)]
        paths: PathArgs,
    },

    /// Lock + git add/commit/push
    Sync {
        #[command(flatten)]
        paths: PathArgs,
        /// Commit message
        #[arg(
            short = 's',
            long,
            default_value = "chore(env): update env with envlock"
        )]
        message: String,
    },
}

// Read envlock.json file to get file paths if config file exists
// Reading the config: cli path >> config.json >> default values
fn load_config_or_defaults() -> (String, String, String) {
    if Path::new(DEFAULT_CONFIG_FILE).exists() {
        let content = fs::read_to_string(DEFAULT_CONFIG_FILE).unwrap();
        let cfg: serde_json::Value = serde_json::from_str(&content).unwrap();

        let env = cfg["envFile"]
            .as_str()
            .unwrap_or(DEFAULT_ENV_FILE)
            .to_string();
        let enc = cfg["encFile"]
            .as_str()
            .unwrap_or(DEFAULT_ENC_FILE)
            .to_string();
        let meta = cfg["metaFile"]
            .as_str()
            .unwrap_or(DEFAULT_META_FILE)
            .to_string();

        return (env, enc, meta);
    }

    (
        DEFAULT_ENV_FILE.to_string(),
        DEFAULT_ENC_FILE.to_string(),
        DEFAULT_META_FILE.to_string(),
    )
}

pub fn run() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let resolve_paths = |env: Option<String>, enc: Option<String>, meta: Option<String>| {
        let (env_path, enc_path, meta_path) = load_config_or_defaults();
        (
            env.unwrap_or(env_path.clone()),
            enc.unwrap_or(enc_path.clone()),
            meta.unwrap_or(meta_path.clone()),
        )
    };

    match cli.command {
        Commands::Lock { paths, force } => {
            let (env, enc, meta) = resolve_paths(paths.env, paths.enc, paths.meta);
            core::cmd_lock(&env, &enc, &meta, force)
        }
        Commands::Unlock { paths, force } => {
            let (env, enc, meta) = resolve_paths(paths.env, paths.enc, paths.meta);
            core::cmd_unlock(&env, &enc, &meta, force)
        }
        Commands::Diff { paths } => {
            let (env, enc, meta) = resolve_paths(paths.env, paths.enc, paths.meta);
            core::cmd_diff(&env, &enc, &meta)
        }
        Commands::Sync { paths, message } => {
            let (env, enc, meta) = resolve_paths(paths.env, paths.enc, paths.meta);
            core::cmd_sync(&env, &enc, &meta, &message)
        }
    }
}
