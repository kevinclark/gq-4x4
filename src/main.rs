use anyhow::{anyhow, Result};
use gq4x4;
use hex;
use pretty_hex::*;
use rusb::{DeviceHandle, UsbContext};
use rustyline::{completion::Completer, Context};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::time::Duration;

fn main() -> Result<()> {
    let mut handle = gq4x4::init()?;

    let mut rl = rustyline::Editor::<ReadlineHelper>::new();
    rl.set_helper(Some(ReadlineHelper {}));

    loop {
        let line = rl.readline(">> ")?;
        rl.add_history_entry(&line);

        let mut parts = line.split(' ');
        if let Some(name) = parts.next() {
            match NAME_TO_COMMAND.iter().find(|(n, _)| *n == name) {
                Some((_, command)) => match command {
                    Command::Quit => return Ok(()),
                    _ => {
                        match run_command(
                            &mut handle,
                            command,
                            &parts.collect(),
                        ) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                },
                None => println!("Unknown command: {}", line),
            }
        } else {
            println!("No line");
        }
    }
}

fn run_command<T: UsbContext>(
    mut handle: &mut DeviceHandle<T>,
    command: &Command,
    args: &Vec<&str>,
) -> Result<String> {
    use Command::*;

    match *command {
        PrintDetails => {
            let details = device_details(&handle)?;
            Ok(format!("{:#?}", details))
        }
        Read => {
            let chunk = gq4x4::read(&mut handle)?;
            let chunk = &chunk.bytes[..chunk.len];
            Ok(format!("{}", pretty_hex(&chunk)))
        }
        FirmwareVersion => {
            let chunk = gq4x4::firmware_version(&mut handle)?;
            let chunk = &chunk.bytes[..chunk.len];
            Ok(format!("{}", pretty_hex(&chunk)))
        }
        SerialNumber => {
            let chunk = gq4x4::serial_number(&mut handle)?;
            let chunk = &chunk.bytes[..chunk.len];
            Ok(format!("{}", pretty_hex(&chunk)))
        }
        Poke => {
            gq4x4::poke(&mut handle, &hex::decode(args.join(""))?)?;
            Ok("Ok".to_string())
        }
        Peek => {
            let chunk = gq4x4::peek(&mut handle)?;
            let chunk = &chunk.bytes[..chunk.len];
            Ok(format!("{}", pretty_hex(&chunk)))
        }
        Quit => panic!("Quit command shouldn't be passed to run_command"),
    }
}

enum Command {
    Poke,
    Peek,
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
    (&"poke", Command::Poke),
    (&"peek", Command::Peek),
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

#[derive(Debug)]
struct DeviceDetails {
    manufacturer: Option<String>,
    product: Option<String>,
    config: u8,
    languages: Vec<rusb::Language>,
}

fn device_details(
    handle: &DeviceHandle<impl UsbContext>,
) -> Result<DeviceDetails> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    if let Some(language) = languages.first() {
        Ok(DeviceDetails {
            manufacturer: handle
                .read_manufacturer_string(*language, &device_desc, timeout)
                .ok(),
            product: handle
                .read_product_string(*language, &device_desc, timeout)
                .ok(),
            config: handle.active_configuration()?,
            languages,
        })
    } else {
        Err(anyhow!("No language found"))
    }
}
