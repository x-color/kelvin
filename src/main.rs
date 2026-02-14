mod cli;
mod commands;
mod config;
mod models;
mod state;
mod storage;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    commands::execute(cli.command)?;
    Ok(())
}
