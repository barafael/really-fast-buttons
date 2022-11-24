use anyhow::Context;
use clap::Parser;
use rfb_client::{interface::Args, process};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    process(args).context("Processing error")?;
    Ok(())
}
