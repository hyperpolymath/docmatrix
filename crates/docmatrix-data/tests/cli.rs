// SPDX-License-Identifier: MPL-2.0
use docmatrix_data::{Format, Output as DataOutput};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn invoke(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_docmatrix"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A usage error exits before reading stdin; that closed pipe is expected.
    if let Err(error) = child.stdin.take().unwrap().write_all(input) {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe, "{error}");
    }
    child.wait_with_output().unwrap()
}

#[test]
fn three_frontends_preserve_the_canonical_and_file_byte_profiles() {
    for (from, input) in [
        ("kyaml", "b: true\na: 2"),
        ("json", "{\"b\":true,\"a\":2}"),
        ("dhall", "{ b = True, a = 2 }"),
    ] {
        for (to, expected) in [
            ("jcs", b"{\"a\":2,\"b\":true}".as_slice()),
            ("json", b"{\"a\":2,\"b\":true}\n".as_slice()),
        ] {
            let out = invoke(
                &["convert", "--from", from, "--to", to, "-"],
                input.as_bytes(),
            );
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert_eq!(out.stdout, expected);
            assert!(out.stderr.is_empty());
        }
    }
}

#[test]
fn bad_input_or_usage_never_produces_partial_output() {
    for (args, input) in [
        (
            vec!["convert", "--from", "kyaml", "--to", "json"],
            "a: 1\na: 2",
        ),
        (
            vec!["convert", "--from", "kyaml", "--to", "jcs"],
            "a: 1\n---\na: 2",
        ),
        (
            vec!["convert", "--from", "dhall", "--to", "json"],
            "env:HOME as Text",
        ),
        (vec!["convert", "--from", "json"], "{}"),
        (vec!["unknown"], ""),
    ] {
        let out = invoke(&args, input.as_bytes());
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_arguments_are_errors_not_panics() {
    use std::os::unix::ffi::OsStringExt;
    let out = Command::new(env!("CARGO_BIN_EXE_docmatrix"))
        .arg(std::ffi::OsString::from_vec(vec![0xff]))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("panicked"));
}

#[test]
fn large_integral_jcs_has_an_explicit_input_policy() {
    let wire = b"10000000000000000";
    let accepted = invoke(&["convert", "--from", "jcs", "--to", "jcs"], wire);
    assert!(accepted.status.success());
    assert_eq!(accepted.stdout, wire);
    let strict = invoke(&["convert", "--from", "json", "--to", "jcs"], wire);
    assert_eq!(strict.status.code(), Some(2));
    assert!(strict.stdout.is_empty());
    let rounded = invoke(
        &["convert", "--from", "jcs", "--to", "jcs"],
        b"9007199254740993",
    );
    assert!(rounded.status.success());
    assert_eq!(rounded.stdout, b"9007199254740992");
}

/// serde_dhall 0.13 is exponential in nesting depth (depth 14 takes ~40 s).
fn deep_dhall(depth: usize) -> String {
    format!("{}1{}", "[".repeat(depth), "]".repeat(depth))
}

#[test]
fn dhall_is_bounded_by_the_wall_clock_deadline() {
    let started = std::time::Instant::now();
    let out = invoke(
        &[
            "convert",
            "--from",
            "dhall",
            "--to",
            "jcs",
            "--dhall-timeout",
            "1",
            "-",
        ],
        deep_dhall(16).as_bytes(),
    );
    let elapsed = started.elapsed();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("RESOURCE_LIMIT"), "{stderr}");
    assert!(elapsed < std::time::Duration::from_secs(6), "{elapsed:?}");
}

#[test]
fn dhall_worker_reports_conversion_errors_unchanged() {
    let input = b"{ a = ";
    let out = invoke(&["convert", "--from", "dhall", "--to", "jcs", "-"], input);
    assert_eq!(out.status.code(), Some(2));
    let direct = docmatrix_data::convert(input, Format::Dhall, DataOutput::Jcs).unwrap_err();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(direct.code), "{stderr}");
    assert!(stderr.contains(&direct.message), "{stderr}");
}

#[test]
fn dhall_timeout_option_is_validated() {
    for value in ["0", "3601", "x", ""] {
        let out = invoke(
            &[
                "convert",
                "--from",
                "dhall",
                "--to",
                "jcs",
                "--dhall-timeout",
                value,
            ],
            b"1",
        );
        assert_eq!(out.status.code(), Some(2), "{value:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("--dhall-timeout"));
    }
}

#[test]
fn worker_protocol_errors_are_reported_not_panicked() {
    let out = invoke(&["__convert-worker", "dhall"], b"");
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("WORKER_PROTOCOL"));
}

#[cfg(unix)]
mod unix_limits {
    use super::*;
    use docmatrix_data::isolate::{convert_isolated, Limits};
    use std::path::Path;
    use std::time::{Duration, Instant};

    fn worker() -> &'static Path {
        Path::new(env!("CARGO_BIN_EXE_docmatrix"))
    }

    /// The wall deadline is far away, so only RLIMIT_CPU can end this run.
    #[test]
    fn cpu_rlimit_ends_the_worker() {
        let limits = Limits {
            wall: Duration::from_secs(120),
            cpu_seconds: 1,
            ..Limits::default()
        };
        let started = Instant::now();
        let error = convert_isolated(
            worker(),
            deep_dhall(16).as_bytes(),
            Format::Dhall,
            DataOutput::Jcs,
            limits,
        )
        .unwrap_err();
        assert_eq!(error.code, "RESOURCE_LIMIT");
        assert!(started.elapsed() < Duration::from_secs(20));
    }

    /// Reading 8 MiB of input cannot fit in a 16 MiB address space.
    #[test]
    fn address_space_rlimit_ends_the_worker() {
        let limits = Limits {
            wall: Duration::from_secs(120),
            address_space_bytes: 16 * 1024 * 1024,
            ..Limits::default()
        };
        let input = vec![b' '; 8 * 1024 * 1024];
        let error =
            convert_isolated(worker(), &input, Format::Json, DataOutput::Jcs, limits).unwrap_err();
        assert_eq!(error.code, "RESOURCE_LIMIT", "{}", error.message);
    }

    #[test]
    fn isolated_and_in_process_conversion_agree() {
        for (input, format) in [
            ("{ b = True, a = [1, 2] }", Format::Dhall),
            ("b: true\na: [1, 2]\n", Format::Kyaml),
        ] {
            assert_eq!(
                convert_isolated(
                    worker(),
                    input.as_bytes(),
                    format,
                    DataOutput::Jcs,
                    Limits::default()
                )
                .unwrap(),
                docmatrix_data::convert(input.as_bytes(), format, DataOutput::Jcs).unwrap()
            );
        }
    }
}
