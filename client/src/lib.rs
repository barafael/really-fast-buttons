use std::io::{Read, Write};

use anyhow::Context;
use interface::{Action, Args};
use linux_embedded_hal::{
    serial_core::{BaudRate, CharSize, FlowControl, Parity, PortSettings, SerialPort, StopBits},
    serial_unix::TTYPort,
};
use rfb_proto::{
    from_bytes, to_vec, ActuatorRequest, ActuatorResponse, SensorRequest, SensorResponse,
};

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
        Action::GetCount => {
            let request = SensorRequest::GetCount;
            let bytes: rfb_proto::Vec<u8, 1> = to_vec(&request).unwrap();
            port.write_all(&bytes).context("Request failed")?;
            let mut response = [0u8; 5];
            port.read_exact(&mut response)
                .context("Receiving response failed")?;
            let response: SensorResponse = from_bytes(&response).expect("Parsing response failed");
            if let SensorResponse::Count(n) = response {
                println!("n = {n}");
            } else {
                eprintln!("Unexpected response: {response:?}");
            }
        }
        Action::Generate {
            rising_edges,
            period_picos,
        } => {
            let request = ActuatorRequest::Generate {
                rising_edges,
                period_picos,
            };
            let bytes: rfb_proto::Vec<u8, 17> = to_vec(&request).unwrap();
            port.write_all(&bytes)
                .context("Writing generator request failed")?;

            // TODO display progressbar and simultaneously wait for any message on `port`
            // Should send `StartedGenerating` or `FailedGenerating`, then for a while nothing,
            // until `FailedGenerating` or `FinishedGenerating`.

            let mut response = [0u8; 1];
            port.read_exact(&mut response)
                .context("Receiving response failed")?;
            let response: ActuatorResponse =
                from_bytes(&response).expect("Parsing response failed");
            loop {
                match response {
                    ActuatorResponse::StartedGenerating => println!("Started generating"),
                    ActuatorResponse::FailedGenerating => {
                        return Err(anyhow::anyhow!("Failed generating"));
                    }
                    ActuatorResponse::FinishedGenerating => {
                        println!("Finished Generating");
                        break;
                    }
                    ActuatorResponse::IAm(_s) => {
                        return Err(anyhow::anyhow!("Unexpected response"));
                    }
                }
            }
        }
        Action::GetDeviceId => {
            let request = SensorRequest::WhoAreYou;
            let bytes: rfb_proto::Vec<u8, 1> = to_vec(&request).unwrap();
            port.write_all(&bytes).context("Request failed")?;
            let mut response = [0u8; 21];
            port.read_exact(&mut response)
                .context("Receiving response failed")?;
            let response: SensorResponse = from_bytes(&response).expect("Parsing response failed");
            if let SensorResponse::IAm(str) = response {
                println!("Id: {str}");
            } else {
                eprintln!("Unexpected response: {response:?}");
            }
        }
    }
    Ok(())
}
