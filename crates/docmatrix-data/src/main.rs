// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
#![forbid(unsafe_code)]
use docmatrix_data::{Format, Output};
use std::io::{Read, Write};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os()
        .skip(1)
        .map(|arg| arg.into_string().map_err(|_| "arguments must be UTF-8"))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();
    let command = args.next().unwrap_or_else(|| "--help".into());
    if command == "--help" {
        println!("docmatrix convert --from kyaml|json|jcs|dhall --to kyaml|json|jcs [FILE|-]\nStructured-data conversion through kyq. Prose conversion remains in the formatrix-core API.");
        return Ok(());
    }
    if command == "--version" {
        println!("docmatrix {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if command != "convert" {
        return Err("expected convert; use --help".into());
    }
    let mut from = None;
    let mut to = None;
    let mut file = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from" => {
                from = Some(match args.next().as_deref() {
                    Some("kyaml" | "yaml") => Format::Kyaml,
                    Some("json") => Format::Json,
                    Some("jcs") => Format::Jcs,
                    Some("dhall") => Format::Dhall,
                    _ => return Err("unsupported input format".into()),
                })
            }
            "--to" => {
                to = Some(match args.next().as_deref() {
                    Some("kyaml" | "yaml") => Output::Kyaml,
                    Some("json") => Output::Json,
                    Some("jcs") => Output::Jcs,
                    _ => return Err("unsupported output format".into()),
                })
            }
            "-" => {
                if file.replace(arg).is_some() {
                    return Err("only one input is accepted".into());
                }
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}").into()),
            _ => {
                if file.replace(arg).is_some() {
                    return Err("only one input is accepted".into());
                }
            }
        }
    }
    let from = from.ok_or("--from is required")?;
    let to = to.ok_or("--to is required")?;
    let reader: Box<dyn Read> = match file.as_deref() {
        None | Some("-") => Box::new(std::io::stdin()),
        Some(path) => Box::new(std::fs::File::open(path)?),
    };
    let mut bytes = Vec::new();
    reader
        .take(kyq::MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    std::io::stdout()
        .lock()
        .write_all(&docmatrix_data::convert(&bytes, from, to)?)?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("docmatrix: {error}");
        std::process::exit(2);
    }
}
