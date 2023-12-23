use clap::{Parser, ValueHint};
use std::path::PathBuf;

/// Interaction with the target sensor or actuator hardware
#[derive(Debug, Parser)]
pub struct Args {
    /// Which action to perform
    #[clap(subcommand)]
    pub action: Action,
}

const DEFAULT_USB_PATH: &str = "/dev/ttyUSB0";

/// An action to be performed
#[derive(Debug, Parser)]
pub enum Action {
    /// Query the value of the current event counter and reset it
    GetCount {
        /// Path to serial port
        #[clap(short, long, parse(from_os_str), default_value = DEFAULT_USB_PATH, value_hint = ValueHint::FilePath,  multiple_occurrences(true))]
        port: Vec<PathBuf>,
    },

    /// Request a number of edges at a period
    Generate {
        /// Path to serial port
        #[clap(short, long, parse(from_os_str), default_value = DEFAULT_USB_PATH, value_hint = ValueHint::FilePath)]
        port: PathBuf,

        /// Number of rising edges
        rising_edges: u32,

        /// picoseconds per period
        period_picos: u64,
    },
}
