// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Structured-data conversion stays separate from the prose document AST.
#![forbid(unsafe_code)]
pub use kyq::{Error, Format, Value};

/// Supported target representations. Dhall is currently an input language.
#[derive(Clone, Copy, Debug)]
pub enum Output {
    /// Deterministic block KYAML, including document streams.
    Kyaml,
    /// A single canonical JSON document followed by LF.
    Json,
    /// RFC 8785 bytes without LF, for hashing and signatures.
    Jcs,
}

/// Convert structured data without routing through a lossy prose format.
/// Comments, anchor spelling and tags are presentation metadata and omitted.
/// Multiple documents cannot silently be collapsed into one JSON document.
pub fn convert(input: &[u8], from: Format, to: Output) -> Result<Vec<u8>, Error> {
    match to {
        Output::Kyaml => kyq::format_documents(&kyq::parse(input, from)?),
        Output::Json | Output::Jcs => {
            let mut output = kyq::canonicalize(input, from)?;
            if matches!(to, Output::Json) {
                output.push(b'\n');
            }
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yaml_dhall_and_json_share_the_same_canonical_boundary() {
        use proven_evidence::parser_contract::{self, Case, Expected};
        for (valid, invalid, from) in [
            ("b: true\na: 2", "a: 1\na: 2", Format::Kyaml),
            (r#"{"b":true,"a":2}"#, r#"{"a":1,"a":2}"#, Format::Json),
            (
                "let x = 1 in { b = True, a = x + 1 }",
                "1 + True",
                Format::Dhall,
            ),
        ] {
            let cases = [
                Case {
                    id: "canonical-object".into(),
                    input: valid.as_bytes().to_vec(),
                    expected: Expected::Canonical(br#"{"a":2,"b":true}"#.to_vec()),
                },
                Case {
                    id: "invalid-input".into(),
                    input: invalid.as_bytes().to_vec(),
                    expected: Expected::Reject,
                },
            ];
            let evidence = parser_contract::run(&cases, |input| {
                convert(input, from, Output::Jcs).map_err(|error| error.to_string())
            })
            .unwrap();
            parser_contract::require_pass(&evidence).unwrap();
        }
    }
    #[test]
    fn malformed_input_and_multidoc_are_not_silently_repaired() {
        for input in ["a: 1\na: 2", "a: 1\n---\na: 2", "a: .inf"] {
            assert!(convert(input.as_bytes(), Format::Kyaml, Output::Json).is_err());
        }
    }
    #[test]
    fn preserve_semantics_across_format_round_trip() {
        let json = br#"{"b":"yes","a":[1,true,null,{"nested":"x"}]}"#;
        let yaml = convert(json, Format::Json, Output::Kyaml).unwrap();
        assert_eq!(
            convert(&yaml, Format::Kyaml, Output::Jcs).unwrap(),
            convert(json, Format::Json, Output::Jcs).unwrap()
        );
    }
}
