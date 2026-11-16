// SPDX-License-Identifier: MPL-2.0
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
    child.stdin.take().unwrap().write_all(input).unwrap();
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
