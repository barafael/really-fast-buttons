use anyhow::{anyhow, Context};
use arguments::{Action, Args};
use linux_embedded_hal::{
    serial_core::{BaudRate, CharSize, FlowControl, Parity, PortSettings, SerialPort, StopBits},
    serial_unix::TTYPort,
};
use rfb_proto::{
    from_bytes, to_vec, ActuatorRequest, ActuatorResponse, SensorRequest, SensorResponse,
};
use std::{
    io::{Read, Write},
    path::Path,
};

pub mod arguments;

pub type Result<T> = anyhow::Result<T>;

const SETTINGS: PortSettings = PortSettings {
    baud_rate: BaudRate::Baud9600,
    char_size: CharSize::Bits8,
    parity: Parity::ParityNone,
    stop_bits: StopBits::Stop1,
    flow_control: FlowControl::FlowNone,
};

fn get_port(path: impl AsRef<Path>) -> Result<TTYPort> {
    let mut port = TTYPort::open(path.as_ref()).map_err(|e| {
        anyhow!(
            "Failed to open tty port \"{}\": {e}",
            path.as_ref().display()
        )
    })?;
    port.configure(&SETTINGS)
        .context("Failed to configure serial port")?;
    Ok(port)
}

fn get_device_id(port: &mut TTYPort) -> Result<String> {
    let request = SensorRequest::WhoAreYou;
    let bytes: rfb_proto::Vec<u8, 1> = to_vec(&request).unwrap();
    port.write_all(&bytes).context("Request failed")?;
    let mut response = [0u8; 21];
    port.read_exact(&mut response)
        .context("Receiving response failed")?;
    let response: SensorResponse = from_bytes(&response).expect("Parsing response failed");
    if let SensorResponse::IAm(str) = response {
        Ok(str.to_string())
    } else {
        Err(anyhow!("Unexpected response"))
    }
}

pub fn process(args: Args) -> Result<()> {
    match args.action {
        Action::GetCount { port: ports } => {
            for port in ports {
                let mut port = get_port(&port)?;
                let id = get_device_id(&mut port)?;
                let request = SensorRequest::GetCount;
                let bytes: rfb_proto::Vec<u8, 1> = to_vec(&request).unwrap();
                port.write_all(&bytes).context("Request failed")?;
                let mut response = [0u8; 5];
                port.read_exact(&mut response)
                    .context("Receiving response failed")?;
                let response: SensorResponse =
                    from_bytes(&response).expect("Parsing response failed");
                if let SensorResponse::Count(n) = response {
                    println!("id {id}: {n}");
                } else {
                    eprintln!("Unexpected response: {response:?}");
                }
            }
        }
        Action::Generate {
            port,
            rising_edges,
            period_picos,
        } => {
            let mut port = get_port(&port)?;
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
                        return Err(anyhow!("Failed generating"));
                    }
                    ActuatorResponse::FinishedGenerating => {
                        println!("Finished Generating");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
