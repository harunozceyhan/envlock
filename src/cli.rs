use crate::core;
use clap::{Parser, Subcommand};

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

#[derive(Subcommand, Debug)]
enum Commands {
    /// Encrypt .env → .env.enc + .env.meta.json
    Lock {
        /// Path to .env file (default: .env)
        #[arg(short, long, default_value = DEFAULT_ENV_FILE)]
        env: String,
        /// Overwrite existing .env.enc without asking
        #[arg(long)]
        force: bool,
    },

    /// Decrypt .env.enc → .env
    Unlock {
        /// Path to encrypted file (default: .env.enc)
        #[arg(short, long, default_value = DEFAULT_ENC_FILE)]
        enc: String,
        /// Overwrite existing .env without asking
        #[arg(long)]
        force: bool,
    },

    /// Diff between current .env and decrypted .env.enc
    Diff {
        /// Path to .env file (default: .env)
        #[arg(short, long, default_value = DEFAULT_ENV_FILE)]
        env: String,
        /// Path to encrypted file (default: .env.enc)
        #[arg(short = 'e', long, default_value = DEFAULT_ENC_FILE)]
        enc: String,
    },

    /// Lock + git add/commit/push
    Sync {
        /// Commit message
        #[arg(short, long, default_value = "chore(env): update env with envlock")]
        message: String,
        /// Path to .env file (default: .env)
        #[arg(short, long, default_value = DEFAULT_ENV_FILE)]
        env: String,
    },
}

pub fn run() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lock { env, force } => {
            core::cmd_lock(&env, DEFAULT_ENC_FILE, DEFAULT_META_FILE, force)
        }
        Commands::Unlock { enc, force } => {
            core::cmd_unlock(DEFAULT_ENV_FILE, &enc, DEFAULT_META_FILE, force)
        }
        Commands::Diff { env, enc } => core::cmd_diff(&env, &enc, DEFAULT_META_FILE),
        Commands::Sync { message, env } => {
            core::cmd_sync(&env, DEFAULT_ENC_FILE, DEFAULT_META_FILE, &message)
        }
    }
}
