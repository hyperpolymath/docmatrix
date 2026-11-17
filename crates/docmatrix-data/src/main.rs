// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
#![forbid(unsafe_code)]
use docmatrix_data::isolate::{convert_isolated, run_worker, Limits, WORKER_ARG};
use docmatrix_data::{Format, Output};
use std::io::{Read, Write};
use std::process::ExitCode;
use std::time::Duration;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os()
        .skip(1)
        .map(|arg| arg.into_string().map_err(|_| "arguments must be UTF-8"))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();
    let command = args.next().unwrap_or_else(|| "--help".into());
    if command == "--help" {
        println!("docmatrix convert --from kyaml|json|jcs|dhall --to kyaml|json|jcs [--dhall-timeout SECONDS] [FILE|-]\nStructured-data conversion through kyq. Dhall is evaluated in a resource-limited worker process (default 10 s). Prose conversion remains in the formatrix-core API.");
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
    let mut limits = Limits::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dhall-timeout" => {
                let seconds = args
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .filter(|seconds| (1..=3600).contains(seconds))
                    .ok_or("--dhall-timeout takes 1 to 3600 seconds")?;
                limits.wall = Duration::from_secs(seconds);
                limits.cpu_seconds = seconds;
            }
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
    let output = if matches!(from, Format::Dhall) {
        convert_isolated(&std::env::current_exe()?, &bytes, from, to, limits)?
    } else {
        docmatrix_data::convert(&bytes, from, to)?
    };
    std::io::stdout().lock().write_all(&output)?;
    Ok(())
}
fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new(WORKER_ARG)) {
        // Non-UTF-8 worker arguments become a protocol error, never a panic.
        let rest: Option<Vec<String>> = args.map(|arg| arg.into_string().ok()).collect();
        return run_worker(&rest.unwrap_or_default());
    }
    if let Err(error) = run() {
        eprintln!("docmatrix: {error}");
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
