// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Metamorphic checks against the real structured-data conversion boundary.
//!
//! [`check`] never panics: every broken property is returned as a
//! [`Violation`]. The fuzz target binaries turn a violation into the abort
//! that libFuzzer records as a finding; tests assert on the value instead.
#![forbid(unsafe_code)]
use docmatrix_data::{Error, Format, Output, convert};
use std::fmt;

/// A metamorphic property of the converter that an input broke.
#[derive(Debug)]
pub enum Violation {
    /// Canonical JCS output was rejected by the explicit JCS reader.
    CanonicalRejected(Error),
    /// Re-canonicalising canonical JCS changed the bytes.
    NotIdempotent { first: Vec<u8>, second: Vec<u8> },
    /// A value accepted for JCS could not be formatted as KYAML for a reason
    /// other than the documented `KEY_LIMIT` target rejection.
    KyamlFormat(Error),
    /// Formatted KYAML was rejected when read back.
    KyamlReparse(Error),
    /// Formatted KYAML read back as a different value.
    KyamlValueChanged { expected: Vec<u8>, actual: Vec<u8> },
    /// A value accepted for JCS was rejected for JSON output.
    JsonRejected(Error),
    /// JSON output was not the canonical bytes followed by exactly one LF.
    JsonFinalLf { expected: Vec<u8>, actual: Vec<u8> },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
        match self {
            Self::CanonicalRejected(e) => {
                write!(f, "canonical JCS must parse with binary64 semantics: {e}")
            }
            Self::NotIdempotent { first, second } => write!(
                f,
                "canonicalization must be idempotent: {:?} != {:?}",
                text(first),
                text(second)
            ),
            Self::KyamlFormat(e) => write!(f, "accepted value must format as KYAML: {e}"),
            Self::KyamlReparse(e) => write!(f, "formatted KYAML must parse: {e}"),
            Self::KyamlValueChanged { expected, actual } => write!(
                f,
                "KYAML formatting must preserve value semantics: {:?} != {:?}",
                text(expected),
                text(actual)
            ),
            Self::JsonRejected(e) => write!(f, "accepted value must format as JSON: {e}"),
            Self::JsonFinalLf { expected, actual } => write!(
                f,
                "JSON output must be canonical bytes plus one LF: {:?} != {:?}",
                text(expected),
                text(actual)
            ),
        }
    }
}

impl std::error::Error for Violation {}

/// Inputs above this size are skipped. The production input limit has
/// separate boundary tests; small mutations give useful parser coverage
/// within the CI budget.
pub const MAX_INPUT: usize = 65_536;

/// In-process Dhall evaluation is exponential in nesting depth (serde_dhall
/// 0.13); depth 8 costs about 50 ms, depth 14 about 40 s. That time is bounded
/// at the process boundary (`docmatrix_data::isolate`, tested in
/// `crates/docmatrix-data/tests/cli.rs`), not here. This target checks value
/// semantics, so deeper Dhall inputs are outside its scope.
pub const MAX_DHALL_NESTING: usize = 8;

/// Deepest bracket nesting in `data`, counting `(`, `[` and `{` everywhere.
/// Brackets inside text literals also count, which only narrows the scope.
pub fn nesting_depth(data: &[u8]) -> usize {
    let (mut depth, mut deepest) = (0usize, 0usize);
    for byte in data {
        match byte {
            b'(' | b'[' | b'{' => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

/// Check the converter's metamorphic properties for one input.
///
/// Rejected input is a valid outcome and returns `Ok(())`. Only an accepted
/// value whose conversions disagree is a [`Violation`].
pub fn check(data: &[u8], from: Format) -> Result<(), Violation> {
    if data.len() > MAX_INPUT {
        return Ok(());
    }
    if matches!(from, Format::Dhall) && nesting_depth(data) > MAX_DHALL_NESTING {
        return Ok(());
    }
    let Ok(canonical) = convert(data, from, Output::Jcs) else {
        return Ok(());
    };
    check_canonical(data, from, canonical)
}

fn check_canonical(data: &[u8], from: Format, canonical: Vec<u8>) -> Result<(), Violation> {
    idempotence(&canonical, convert(&canonical, Format::Jcs, Output::Jcs))?;
    kyaml_round_trip(&canonical, convert(data, from, Output::Kyaml), |yaml| {
        convert(yaml, Format::Kyaml, Output::Jcs)
    })?;
    json_final_lf(canonical, convert(data, from, Output::Json))
}

/// Re-reading canonical JCS through the explicit JCS reader is the identity.
fn idempotence(canonical: &[u8], again: Result<Vec<u8>, Error>) -> Result<(), Violation> {
    let again = again.map_err(Violation::CanonicalRejected)?;
    if again == canonical {
        Ok(())
    } else {
        Err(Violation::NotIdempotent {
            first: canonical.to_vec(),
            second: again,
        })
    }
}

/// Values that JSON can hold but the KYAML contract cannot: implicit keys over
/// 1024 characters (`KEY_LIMIT`, `KEY_TOO_LONG`) and the YAML 1.1 merge key
/// `<<`, which kyq refuses in every spelling (`UNSUPPORTED_MERGE_KEY`).
pub const KYAML_TARGET_REJECTIONS: &[&str] =
    &["KEY_LIMIT", "KEY_TOO_LONG", "UNSUPPORTED_MERGE_KEY"];

/// Formatted KYAML reads back as the same value. A documented target
/// rejection is not a violation.
fn kyaml_round_trip(
    canonical: &[u8],
    formatted: Result<Vec<u8>, Error>,
    reread: impl FnOnce(&[u8]) -> Result<Vec<u8>, Error>,
) -> Result<(), Violation> {
    let yaml = match formatted {
        Ok(yaml) => yaml,
        Err(error) if KYAML_TARGET_REJECTIONS.contains(&error.code) => return Ok(()),
        Err(error) => return Err(Violation::KyamlFormat(error)),
    };
    let actual = reread(&yaml).map_err(Violation::KyamlReparse)?;
    if actual == canonical {
        Ok(())
    } else {
        Err(Violation::KyamlValueChanged {
            expected: canonical.to_vec(),
            actual,
        })
    }
}

/// JSON output is the canonical bytes followed by exactly one LF.
fn json_final_lf(canonical: Vec<u8>, json: Result<Vec<u8>, Error>) -> Result<(), Violation> {
    let actual = json.map_err(Violation::JsonRejected)?;
    let mut expected = canonical;
    expected.push(b'\n');
    if actual == expected {
        Ok(())
    } else {
        Err(Violation::JsonFinalLf { expected, actual })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kyaml_key_limit_is_an_expected_rejection() {
        for key in ["a".repeat(1025), r"\n".repeat(512)] {
            let json = format!(r#"{{"{key}":0}}"#);
            assert_eq!(
                convert(json.as_bytes(), Format::Json, Output::Kyaml)
                    .unwrap_err()
                    .code,
                "KEY_LIMIT"
            );
            check(json.as_bytes(), Format::Json).unwrap();
        }
    }

    #[test]
    fn seeds_reach_the_converter_and_preserve_values() {
        for (data, format) in [
            (
                include_bytes!("../corpus/kyaml/object").as_slice(),
                Format::Kyaml,
            ),
            (
                include_bytes!("../corpus/json/object").as_slice(),
                Format::Json,
            ),
            (
                include_bytes!("../corpus/dhall/object").as_slice(),
                Format::Dhall,
            ),
        ] {
            assert_eq!(
                convert(data, format, Output::Jcs).unwrap(),
                br#"{"a":[1,2,3],"b":"text"}"#
            );
            check(data, format).unwrap();
        }
    }
    #[test]
    fn numeric_wire_seeds_use_the_explicit_jcs_reader() {
        for (data, format) in [
            (
                include_bytes!("../corpus/kyaml/integral_float").as_slice(),
                Format::Kyaml,
            ),
            (
                include_bytes!("../corpus/json/integral_float").as_slice(),
                Format::Json,
            ),
            (
                include_bytes!("../corpus/dhall/integral_float").as_slice(),
                Format::Dhall,
            ),
            (
                include_bytes!("../corpus/jcs/large_integral").as_slice(),
                Format::Jcs,
            ),
            (
                include_bytes!("../corpus/jcs/rounding").as_slice(),
                Format::Jcs,
            ),
        ] {
            let canonical = convert(data, format, Output::Jcs).unwrap();
            assert!(convert(&canonical, Format::Json, Output::Jcs).is_err());
            check(data, format).unwrap();
        }
    }
    #[test]
    fn invalid_seeds_reach_rejection() {
        for (data, format) in [
            (
                include_bytes!("../corpus/kyaml/duplicate").as_slice(),
                Format::Kyaml,
            ),
            (
                include_bytes!("../corpus/json/duplicate").as_slice(),
                Format::Json,
            ),
            (
                include_bytes!("../corpus/dhall/invalid").as_slice(),
                Format::Dhall,
            ),
        ] {
            assert!(convert(data, format, Output::Jcs).is_err());
            check(data, format).unwrap();
        }
    }

    fn error(code: &'static str) -> Error {
        Error {
            code,
            message: "injected".into(),
            line: 1,
            column: 1,
            path: String::new(),
        }
    }

    #[test]
    fn idempotence_reports_rejection_and_drift() {
        assert!(idempotence(b"1", Ok(b"1".to_vec())).is_ok());
        assert!(matches!(
            idempotence(b"1", Err(error("SYNTAX"))),
            Err(Violation::CanonicalRejected(e)) if e.code == "SYNTAX"
        ));
        assert!(matches!(
            idempotence(b"1", Ok(b"2".to_vec())),
            Err(Violation::NotIdempotent { first, second }) if first == b"1" && second == b"2"
        ));
    }

    #[test]
    fn kyaml_round_trip_reports_each_failure() {
        let ok = |_: &[u8]| Ok(b"1".to_vec());
        assert!(kyaml_round_trip(b"1", Ok(b"1\n".to_vec()), ok).is_ok());
        for code in KYAML_TARGET_REJECTIONS {
            assert!(kyaml_round_trip(b"1", Err(error(code)), ok).is_ok());
        }
        assert!(matches!(
            kyaml_round_trip(b"1", Err(error("DEPTH")), ok),
            Err(Violation::KyamlFormat(e)) if e.code == "DEPTH"
        ));
        assert!(matches!(
            kyaml_round_trip(b"1", Ok(b"1\n".to_vec()), |_| Err(error("SYNTAX"))),
            Err(Violation::KyamlReparse(e)) if e.code == "SYNTAX"
        ));
        assert!(matches!(
            kyaml_round_trip(b"1", Ok(b"2\n".to_vec()), |_| Ok(b"2".to_vec())),
            Err(Violation::KyamlValueChanged { expected, actual })
                if expected == b"1" && actual == b"2"
        ));
    }

    #[test]
    fn json_final_lf_reports_rejection_and_framing() {
        assert!(json_final_lf(b"1".to_vec(), Ok(b"1\n".to_vec())).is_ok());
        assert!(matches!(
            json_final_lf(b"1".to_vec(), Err(error("SYNTAX"))),
            Err(Violation::JsonRejected(e)) if e.code == "SYNTAX"
        ));
        for wrong in [b"1".as_slice(), b"1\n\n", b"1\r\n"] {
            assert!(matches!(
                json_final_lf(b"1".to_vec(), Ok(wrong.to_vec())),
                Err(Violation::JsonFinalLf { expected, actual })
                    if expected == b"1\n" && actual == wrong
            ));
        }
    }

    #[test]
    fn violations_propagate_through_the_real_converter() {
        // A wrong canonical value injected after the real JCS step must surface
        // as a value-change violation from the real KYAML round trip.
        let violation =
            check_canonical(br#"{"a":1}"#, Format::Json, br#"{"a":2}"#.to_vec()).unwrap_err();
        assert!(matches!(violation, Violation::KyamlValueChanged { .. }));
        assert!(violation.to_string().contains("preserve value semantics"));
        assert!(matches!(
            check_canonical(b"1", Format::Json, b"not canonical".to_vec()),
            Err(Violation::CanonicalRejected(_))
        ));
    }

    #[test]
    fn oversized_and_rejected_inputs_are_not_violations() {
        assert!(check(&vec![b' '; MAX_INPUT + 1], Format::Json).is_ok());
        assert!(check(b"{", Format::Json).is_ok());
    }

    #[test]
    fn regression_inputs_are_handled_without_panicking() {
        // Every file under regressions/<format>/ is replayed; adding a crash
        // input there is enough to keep it covered.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("regressions");
        let mut replayed = 0;
        for (dir, format) in [
            ("kyaml", Format::Kyaml),
            ("json", Format::Json),
            ("jcs", Format::Jcs),
            ("dhall", Format::Dhall),
        ] {
            for entry in std::fs::read_dir(root.join(dir)).unwrap() {
                let path = entry.unwrap().path();
                let data = std::fs::read(&path).unwrap();
                if let Err(violation) = check(&data, format) {
                    panic!("{}: {violation}", path.display());
                }
                replayed += 1;
            }
        }
        assert_eq!(replayed, 9);
    }

    #[test]
    fn dhall_scope_is_nesting_depth_only() {
        assert_eq!(nesting_depth(b""), 0);
        assert_eq!(nesting_depth(b"[[1]] { a = (2) }"), 2);
        assert_eq!(nesting_depth(b"]]]][[ ]"), 2);
        let deep = format!("{}1{}", "[".repeat(9), "]".repeat(9));
        assert!(check(deep.as_bytes(), Format::Dhall).is_ok());
        // The same depth is still checked for the other front ends.
        let json = format!("{}1{}", "[".repeat(9), "]".repeat(9));
        assert!(check(json.as_bytes(), Format::Json).is_ok());
        assert_eq!(nesting_depth(json.as_bytes()), 9);
    }
}
