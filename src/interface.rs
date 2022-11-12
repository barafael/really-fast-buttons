use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to serial port
    #[clap(short, long, parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub port: PathBuf,

    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Parser)]
pub enum Action {
    Read,
}
