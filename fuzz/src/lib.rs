// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Metamorphic checks against the real structured-data conversion boundary.
#![forbid(unsafe_code)]
use docmatrix_data::{Format, Output, convert};

pub fn check(data: &[u8], from: Format) {
    // The production input limit has separate boundary tests. Keep individual
    // mutations small enough for useful parser coverage within the CI budget.
    if data.len() > 65_536 {
        return;
    }
    let Ok(canonical) = convert(data, from, Output::Jcs) else {
        return;
    };
    assert_eq!(
        convert(&canonical, Format::Jcs, Output::Jcs)
            .expect("canonical JCS must parse with binary64 semantics"),
        canonical,
        "canonicalization must be idempotent"
    );
    match convert(data, from, Output::Kyaml) {
        Ok(yaml) => assert_eq!(
            convert(&yaml, Format::Kyaml, Output::Jcs).expect("formatted KYAML must parse"),
            canonical,
            "KYAML formatting must preserve value semantics"
        ),
        Err(error) if error.code == "KEY_LIMIT" => {}
        Err(error) => panic!("accepted value must format: {error}"),
    }
    let mut line = canonical;
    line.push(b'\n');
    assert_eq!(convert(data, from, Output::Json).unwrap(), line);
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
            check(json.as_bytes(), Format::Json);
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
            check(data, format);
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
            check(data, format);
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
            check(data, format);
        }
    }
}
