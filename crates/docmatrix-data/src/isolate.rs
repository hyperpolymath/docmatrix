// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Conversion in a separate, resource-limited worker process.
//!
//! The Dhall front end (serde_dhall 0.13) takes time exponential in nesting
//! depth, even for valid input: depth 10 takes about 0.3 s and depth 14 about
//! 38 s. A thread cannot be stopped from outside, so untrusted Dhall runs in a
//! child process instead:
//!
//! * the parent enforces a wall-clock deadline and kills the child, on every
//!   platform;
//! * on Unix the child also lowers its own CPU-time limit before reading
//!   input, so it cannot outlive its budget even if the parent disappears;
//! * on Linux it lowers its address-space limit too. macOS rejects lowering
//!   `RLIMIT_AS` (`EINVAL`), so there the memory budget is not enforced.
//!
//! Any limit that ends the worker is reported as `RESOURCE_LIMIT`. Errors from
//! the conversion itself cross the boundary unchanged.
use crate::{convert, Error, Format, Output};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

/// First argument that turns the `docmatrix` executable into a worker.
pub const WORKER_ARG: &str = "__convert-worker";

/// Worker exit status for a conversion error written to stderr.
const CONVERSION_ERROR: u8 = 3;

/// Resource budget for one isolated conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Wall-clock deadline enforced by the parent.
    pub wall: Duration,
    /// CPU seconds the worker may use (`RLIMIT_CPU`, all Unix targets).
    pub cpu_seconds: u64,
    /// Address space the worker may map (`RLIMIT_AS`, Linux only).
    pub address_space_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            wall: Duration::from_secs(10),
            cpu_seconds: 10,
            address_space_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

fn format_name(format: Format) -> &'static str {
    match format {
        Format::Kyaml => "kyaml",
        Format::Json => "json",
        Format::Jcs => "jcs",
        Format::Dhall => "dhall",
    }
}

fn output_name(output: Output) -> &'static str {
    match output {
        Output::Kyaml => "kyaml",
        Output::Json => "json",
        Output::Jcs => "jcs",
    }
}

fn parse_format(name: &str) -> Option<Format> {
    Some(match name {
        "kyaml" => Format::Kyaml,
        "json" => Format::Json,
        "jcs" => Format::Jcs,
        "dhall" => Format::Dhall,
        _ => return None,
    })
}

fn parse_output(name: &str) -> Option<Output> {
    Some(match name {
        "kyaml" => Output::Kyaml,
        "json" => Output::Json,
        "jcs" => Output::Jcs,
        _ => return None,
    })
}

fn error(code: &'static str, message: impl Into<String>) -> Error {
    Error {
        code,
        message: message.into(),
        line: 1,
        column: 1,
        path: String::new(),
    }
}

/// `Error::code` is `&'static str`. Codes arriving from a worker are interned
/// so each distinct code is allocated once per process, however many errors
/// cross the boundary.
fn intern(code: &str) -> &'static str {
    static CODES: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut codes = CODES
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(known) = codes.get(code) {
        return known;
    }
    let leaked: &'static str = Box::leak(code.to_owned().into_boxed_str());
    codes.insert(leaked);
    leaked
}

fn encode_error(e: &Error) -> String {
    let mut value = crate::Value::Object(Default::default());
    if let crate::Value::Object(map) = &mut value {
        map.insert("code".into(), e.code.into());
        map.insert("message".into(), e.message.clone().into());
        map.insert("line".into(), e.line.into());
        map.insert("column".into(), e.column.into());
        map.insert("path".into(), e.path.clone().into());
    }
    value.to_string()
}

fn decode_error(bytes: &[u8]) -> Option<Error> {
    let documents = kyq::parse(bytes, Format::Json).ok()?;
    let crate::Value::Object(map) = documents.first()? else {
        return None;
    };
    let text = |key: &str| map.get(key)?.as_str().map(str::to_owned);
    let number = |key: &str| usize::try_from(map.get(key)?.as_u64()?).ok();
    Some(Error {
        code: intern(&text("code")?),
        message: text("message")?,
        line: number("line")?,
        column: number("column")?,
        path: text("path")?,
    })
}

/// Convert `input` in a worker process started from `worker`, an executable
/// whose `main` hands [`WORKER_ARG`] invocations to [`run_worker`].
pub fn convert_isolated(
    worker: &Path,
    input: &[u8],
    from: Format,
    to: Output,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    let mut child = Command::new(worker)
        .arg(WORKER_ARG)
        .arg(format_name(from))
        .arg(output_name(to))
        .arg(limits.cpu_seconds.to_string())
        .arg(limits.address_space_bytes.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| error("WORKER_START", format!("cannot start worker: {e}")))?;
    let (Some(mut stdin), Some(mut stdout), Some(mut stderr)) =
        (child.stdin.take(), child.stdout.take(), child.stderr.take())
    else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error("WORKER_START", "worker pipes are unavailable"));
    };
    // Feed and drain on separate threads so a full pipe cannot deadlock the
    // parent; a broken pipe only means the worker stopped reading.
    let input = input.to_vec();
    let feeder = thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let out = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let err = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let deadline = Instant::now() + limits.wall;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error("WORKER_WAIT", format!("cannot wait for worker: {e}")));
            }
        }
    };
    let _ = feeder.join();
    let stdout = out.join().ok().and_then(Result::ok).unwrap_or_default();
    let stderr = err.join().ok().and_then(Result::ok).unwrap_or_default();
    let limit = || {
        error(
            "RESOURCE_LIMIT",
            format!(
                "conversion exceeded its budget ({} ms wall, {} s CPU, {} bytes address space)",
                limits.wall.as_millis(),
                limits.cpu_seconds,
                limits.address_space_bytes
            ),
        )
    };
    let Some(status) = status else {
        return Err(limit());
    };
    match status.code() {
        Some(0) => Ok(stdout),
        Some(code) if code == i32::from(CONVERSION_ERROR) => Err(decode_error(&stderr)
            .unwrap_or_else(|| error("WORKER_PROTOCOL", "worker reported an unreadable error"))),
        // A signal (SIGXCPU, SIGKILL) or an allocation failure abort.
        _ => Err(limit()),
    }
}

#[cfg(unix)]
fn apply_limits(cpu_seconds: u64, address_space_bytes: u64) -> Result<(), String> {
    use rlimit::Resource;
    Resource::CPU
        .set(cpu_seconds, cpu_seconds)
        .map_err(|e| format!("RLIMIT_CPU: {e}"))?;
    #[cfg(target_os = "linux")]
    Resource::AS
        .set(address_space_bytes, address_space_bytes)
        .map_err(|e| format!("RLIMIT_AS: {e}"))?;
    #[cfg(not(target_os = "linux"))]
    let _ = address_space_bytes;
    Ok(())
}

#[cfg(not(unix))]
fn apply_limits(_cpu_seconds: u64, _address_space_bytes: u64) -> Result<(), String> {
    // The parent's wall-clock deadline is the bound on this platform.
    Ok(())
}

/// Worker entry point. `args` are the arguments after [`WORKER_ARG`].
pub fn run_worker(args: &[String]) -> ExitCode {
    let fail = |e: Error| {
        let _ = std::io::stderr().write_all(encode_error(&e).as_bytes());
        ExitCode::from(CONVERSION_ERROR)
    };
    let [from, to, cpu, memory] = args else {
        return fail(error(
            "WORKER_PROTOCOL",
            "expected FROM TO CPU_SECONDS AS_BYTES",
        ));
    };
    let (Some(from), Some(to), Ok(cpu), Ok(memory)) = (
        parse_format(from),
        parse_output(to),
        cpu.parse::<u64>(),
        memory.parse::<u64>(),
    ) else {
        return fail(error("WORKER_PROTOCOL", "invalid worker arguments"));
    };
    if let Err(e) = apply_limits(cpu, memory) {
        return fail(error("WORKER_LIMITS", format!("cannot apply {e}")));
    }
    let mut input = Vec::new();
    if let Err(e) = std::io::stdin()
        .take(kyq::MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut input)
    {
        // Allocation failure under RLIMIT_AS surfaces here as OutOfMemory.
        let code = if e.kind() == std::io::ErrorKind::OutOfMemory {
            "RESOURCE_LIMIT"
        } else {
            "WORKER_INPUT"
        };
        return fail(error(code, e.to_string()));
    }
    match convert(&input, from, to) {
        Ok(bytes) => match std::io::stdout().lock().write_all(&bytes) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => fail(error("WORKER_OUTPUT", e.to_string())),
        },
        Err(e) => fail(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_cross_the_boundary_unchanged() {
        let original = Error {
            code: "BAD_ESCAPE",
            message: "line\n\t\"quoted\" \u{feff}".into(),
            line: 7,
            column: 42,
            path: "a/b~1c".into(),
        };
        let decoded = decode_error(encode_error(&original).as_bytes()).unwrap();
        assert_eq!(
            (
                decoded.code,
                decoded.message,
                decoded.line,
                decoded.column,
                decoded.path
            ),
            (
                original.code,
                original.message,
                original.line,
                original.column,
                original.path
            )
        );
    }

    #[test]
    fn interned_codes_are_shared() {
        let a = intern(&String::from("SOME_CODE"));
        let b = intern(&String::from("SOME_CODE"));
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn malformed_worker_reports_are_rejected() {
        for bytes in [
            b"".as_slice(),
            b"not json",
            br#"[]"#,
            br#"{"code":"X"}"#,
            br#"{"code":1,"message":"m","line":1,"column":1,"path":""}"#,
            br#"{"code":"X","message":"m","line":-1,"column":1,"path":""}"#,
        ] {
            assert!(
                decode_error(bytes).is_none(),
                "{:?}",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    #[test]
    fn names_round_trip() {
        for format in [Format::Kyaml, Format::Json, Format::Jcs, Format::Dhall] {
            assert!(matches!(
                (parse_format(format_name(format)), format),
                (Some(Format::Kyaml), Format::Kyaml)
                    | (Some(Format::Json), Format::Json)
                    | (Some(Format::Jcs), Format::Jcs)
                    | (Some(Format::Dhall), Format::Dhall)
            ));
        }
        for output in [Output::Kyaml, Output::Json, Output::Jcs] {
            assert_eq!(
                output_name(parse_output(output_name(output)).unwrap()),
                output_name(output)
            );
        }
    }

    #[test]
    fn missing_worker_is_an_error() {
        let error = convert_isolated(
            Path::new("/nonexistent/docmatrix-worker"),
            b"1",
            Format::Json,
            Output::Jcs,
            Limits::default(),
        )
        .unwrap_err();
        assert_eq!(error.code, "WORKER_START");
    }
}
