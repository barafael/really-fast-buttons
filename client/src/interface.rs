use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to serial port
    #[clap(short, long, parse(from_os_str), default_value = "/dev/ttyUSB0", value_hint = ValueHint::FilePath)]
    pub port: PathBuf,

    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Parser)]
pub enum Action {
    GetCount,
    Generate {
        rising_edges: u32,
        period_picos: u64,
    },
    GetDeviceId,
}
