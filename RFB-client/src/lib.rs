use std::io::{Read, Write};

use anyhow::Context;
use interface::{Action, Args};
use linux_embedded_hal::{
    serial_core::{BaudRate, CharSize, FlowControl, Parity, PortSettings, SerialPort, StopBits},
    serial_unix::TTYPort,
};
use rfb_proto::{from_bytes, to_vec, ActuatorMessage, SensorMessage};

pub mod interface;

const SETTINGS: PortSettings = PortSettings {
    baud_rate: BaudRate::Baud9600,
    char_size: CharSize::Bits8,
    parity: Parity::ParityNone,
    stop_bits: StopBits::Stop1,
    flow_control: FlowControl::FlowNone,
};

pub fn process(args: Args) -> anyhow::Result<()> {
    let mut port = TTYPort::open(&args.port).context("Failed to open tty port")?;
    port.configure(&SETTINGS)
        .expect("Failed to configure serial port");

    match args.action {
        Action::Read => {
            let request = SensorMessage::Request;
            let bytes: rfb_proto::Vec<u8, 9> = to_vec(&request).unwrap();
            port.write_all(&bytes).context("Request failed")?;
            let mut response = [0u8; 9];
            port.read_exact(&mut response)
                .context("Receiving response failed")?;
            let response: SensorMessage = from_bytes(&response).expect("Parsing response failed");
            if let SensorMessage::Response(n) = response {
                println!("n = {n}");
            } else {
                eprintln!("Unexpected response: {response:?}");
            }
        }
        Action::Generate {
            rising_edges,
            period_picos,
        } => {
            let request = ActuatorMessage::Request {
                rising_edges,
                period_picos,
            };
            let bytes: rfb_proto::Vec<u8, 17> = to_vec(&request).unwrap();
            port.write_all(&bytes)
                .context("Writing generator request failed")?;
        }
    }
    Ok(())
}
