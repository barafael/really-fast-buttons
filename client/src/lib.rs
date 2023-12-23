use anyhow::{anyhow, Context};
use arguments::{Action, Args};
use rfb_proto::{
    from_bytes, to_vec, ActuatorRequest, ActuatorResponse, SensorRequest, SensorResponse,
};
use serialport::SerialPort;
use std::{
    io::{Read, Write},
    path::Path,
    time::Duration,
};

pub mod arguments;

pub type Result<T> = anyhow::Result<T>;

fn get_port(path: impl AsRef<Path>) -> Result<Box<dyn SerialPort>> {
    let port = serialport::new(path.as_ref().to_string_lossy(), 9600)
        .timeout(Duration::from_millis(100))
        .open()
        .with_context(|| format!("Failed to open tty port \"{}\"", path.as_ref().display()))?;
    Ok(port)
}

fn get_device_id(port: &mut Box<dyn SerialPort>) -> Result<String> {
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
                let id = {
                    //get_device_id(&mut port)?;
                    "placeholder".to_string()
                };
                let request = SensorRequest::GetCount;
                let bytes: rfb_proto::Vec<u8, 1> = to_vec(&request).unwrap();
                port.write_all(&bytes).context("Request failed")?;
                let mut response = [0u8; 5];
                port.read_exact(&mut response)
                    .context("Receiving response failed")?;
                let response: SensorResponse =
                    from_bytes(&response).expect("Parsing response failed");
                if let SensorResponse::Count(n) = response {
                    println!("id \"{id}\": {n}");
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
