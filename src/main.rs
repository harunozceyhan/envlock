use anyhow::Result;

mod cli;
mod core;
mod utils;

fn main() -> Result<()> {
    cli::run()
}
