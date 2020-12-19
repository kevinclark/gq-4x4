use gq4x4;
use pretty_hex::*;
use rusb::{Device, DeviceHandle, Result, UsbContext};
use rustyline::{completion::Completer, Context};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::time::Duration;

fn main() -> rustyline::Result<()> {
    use Command::*;

    let (_, mut handle) = gq4x4::init().unwrap();

    let mut rl = rustyline::Editor::<ReadlineHelper>::new();
    rl.set_helper(Some(ReadlineHelper {}));

    loop {
        match rl.readline(">> ") {
            Err(e) => return Err(e),
            Ok(line) => {
                match NAME_TO_COMMAND.iter().find(|(n, _)| *n == line) {
                    Some((_, command)) => match command {
                        Quit => return Ok(()),
                        PrintDetails => print_device_info(&handle).unwrap(),
                        Read => {
                            let chunk = gq4x4::read(&mut handle);
                            let chunk = &chunk.bytes[..chunk.len];
                            println!("{}", pretty_hex(&chunk))
                        }
                        FirmwareVersion => {
                            let chunk = gq4x4::firmware_version(&mut handle);
                            let chunk = &chunk.bytes[..chunk.len];
                            println!("{}", pretty_hex(&chunk))
                        }
                        SerialNumber => {
                            let chunk = gq4x4::serial_number(&mut handle);
                            let chunk = &chunk.bytes[..chunk.len];
                            println!("{}", pretty_hex(&chunk))
                        }
                    },
                    None => println!("Unknown command: {}", line),
                }
            }
        }
    }
}

enum Command {
    SerialNumber,
    FirmwareVersion,
    Read,
    PrintDetails,
    Quit,
}

static NAME_TO_COMMAND: &[(&'static str, Command)] = &[
    (&"details", Command::PrintDetails),
    (&"read", Command::Read),
    (&"quit", Command::Quit),
    (&"firmware", Command::FirmwareVersion),
    (&"serial", Command::SerialNumber),
];

#[derive(Helper, Hinter, Highlighter, Validator)]
struct ReadlineHelper;

impl Completer for ReadlineHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if line.contains(' ') || line.len() != pos {
            return Ok((0, vec![]));
        }

        let candidates: Vec<String> = NAME_TO_COMMAND
            .iter()
            .filter(|(n, _)| n.starts_with(line))
            .map(|(n, _)| n.to_string())
            .collect();

        Ok((0, candidates))
    }
}

#[cfg(test)]
mod tests {
    //#[test]
    //fn parse() {
    //assert_eq!(Command::Initialize, parse("initialize"))
    //}
}

fn print_device_info(handle: &DeviceHandle<impl UsbContext>) -> Result<()> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    println!("Available languages: {:#?}", languages);

    if !languages.is_empty() {
        let language = languages[0];
        println!("Language: {:?}", language);

        println!(
            "Manufacturer: {}",
            handle
                .read_manufacturer_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
        println!(
            "Product: {}",
            handle
                .read_product_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
        println!(
            "Serial Number: {}",
            handle
                .read_serial_number_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
    }
    Ok(())
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn find_readable_endpoints<T: UsbContext>(
    device: &Device<T>,
) -> Result<Vec<Endpoint>> {
    let device_desc = device.device_descriptor()?;
    let mut endpoints = vec![];
    for n in 0..device_desc.num_configurations() {
        let config = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    endpoints.push(Endpoint {
                        config: config.number(),
                        iface: interface_desc.interface_number(),
                        setting: interface_desc.setting_number(),
                        address: endpoint_desc.address(),
                    });
                }
            }
        }
    }

    Ok(endpoints)
}
