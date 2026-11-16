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
        Output::Kyaml => {
            let documents = kyq::parse(input, from)?;
            for document in &documents {
                validate_kyaml_keys(document)?;
            }
            let mut output = Vec::new();
            for (index, document) in documents.iter().enumerate() {
                if index > 0 {
                    output.extend_from_slice(b"---\n");
                }
                // kyq's emitter treats this root scalar as a document-end marker.
                if matches!(document, Value::String(value) if value == "...") {
                    output.extend_from_slice(b"\"...\"\n");
                } else {
                    output.extend(kyq::format_documents(std::slice::from_ref(document))?);
                }
            }
            Ok(output)
        }
        Output::Json | Output::Jcs => {
            let mut output = kyq::canonicalize(input, from)?;
            if matches!(to, Output::Json) {
                output.push(b'\n');
            }
            Ok(output)
        }
    }
}

fn validate_kyaml_keys(value: &Value) -> Result<(), Error> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key.chars().count() > 1024 {
                    return Err(Error {
                        code: "KEY_LIMIT",
                        message: "KYAML implicit keys cannot exceed 1024 characters".into(),
                        line: 1,
                        column: 1,
                        path: key.clone(),
                    });
                }
                validate_kyaml_keys(value)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_kyaml_keys(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlong_implicit_keys_are_rejected() {
        let key = "a".repeat(1025);
        for json in [
            format!(r#"{{"{key}":0}}"#),
            format!(r#"[{{"nested":{{"{key}":0}}}}]"#),
        ] {
            let error = convert(json.as_bytes(), Format::Json, Output::Kyaml).unwrap_err();
            assert_eq!(error.code, "KEY_LIMIT");
        }
        let json = format!(r#"{{"{}":0}}"#, "a".repeat(1024));
        assert!(convert(json.as_bytes(), Format::Json, Output::Kyaml).is_ok());
    }

    #[test]
    fn document_end_scalar_round_trips() {
        let json = br#""...""#;
        let yaml = convert(json, Format::Json, Output::Kyaml).unwrap();
        assert_eq!(yaml, b"\"...\"\n");
        assert_eq!(convert(&yaml, Format::Kyaml, Output::Jcs).unwrap(), json);
    }

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
