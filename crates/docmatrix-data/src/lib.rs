// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Structured-data conversion stays separate from the prose document AST.
#![forbid(unsafe_code)]
pub use kyq::{Error, Format, Value};

pub mod isolate;

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
///
/// Dhall evaluation time is not bounded here: serde_dhall 0.13 is exponential
/// in nesting depth. Untrusted Dhall belongs in [`isolate::convert_isolated`].
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
                // Render through the same emitter used for mapping keys so quotes
                // and JSON escapes count toward the reader's implicit-key limit.
                // A scalar rendering includes one trailing LF.
                let rendered = kyq::format_documents(&[Value::String(key.clone())])?;
                if rendered_key_length(&rendered, key)? > 1024 {
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

/// Character length of an emitted scalar key without its trailing LF.
/// Malformed emitter output is reported, never assumed away.
fn rendered_key_length(rendered: &[u8], key: &str) -> Result<usize, Error> {
    let emitter_error = |message: &str| Error {
        code: "EMITTER_OUTPUT",
        message: message.into(),
        line: 1,
        column: 1,
        path: key.to_owned(),
    };
    let text = std::str::from_utf8(rendered)
        .map_err(|_| emitter_error("KYAML emitter produced non-UTF-8 key bytes"))?;
    let scalar = text
        .strip_suffix('\n')
        .ok_or_else(|| emitter_error("KYAML emitter omitted the scalar's trailing LF"))?;
    Ok(scalar.chars().count())
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
    fn emitted_key_boundaries_include_quotes_and_escapes() {
        for (key_json, accepted) in [
            (format!("#{}", "a".repeat(1021)), true),
            (format!("#{}", "a".repeat(1022)), false),
            // Quotes force a quoted key and escape to two characters.
            (r#"\""#.repeat(511), true),
            (r#"\""#.repeat(512), false),
            ("é".repeat(1024), true),
        ] {
            let json = format!(r#"{{"{key_json}":0}}"#);
            let result = convert(json.as_bytes(), Format::Json, Output::Kyaml);
            if accepted {
                let yaml = result.unwrap();
                assert_eq!(
                    convert(&yaml, Format::Kyaml, Output::Jcs).unwrap(),
                    convert(json.as_bytes(), Format::Json, Output::Jcs).unwrap()
                );
            } else {
                assert_eq!(result.unwrap_err().code, "KEY_LIMIT");
            }
        }
    }

    #[test]
    fn escaped_key_limit_is_a_single_cut_off() {
        // How a control character is spelled is the emitter's choice; the
        // contract is that every key either round-trips or is rejected with
        // KEY_LIMIT, and that acceptance stops at exactly one length.
        let mut accepted = Vec::new();
        for count in 1..=600 {
            let json = format!(r#"{{"{}":0}}"#, r"\n".repeat(count));
            match convert(json.as_bytes(), Format::Json, Output::Kyaml) {
                Ok(yaml) => {
                    assert_eq!(
                        convert(&yaml, Format::Kyaml, Output::Jcs).unwrap(),
                        convert(json.as_bytes(), Format::Json, Output::Jcs).unwrap()
                    );
                    accepted.push(count);
                }
                Err(error) => assert_eq!(error.code, "KEY_LIMIT", "{count}"),
            }
        }
        let cut_off = accepted.len();
        assert!(cut_off > 0 && cut_off < 600);
        assert_eq!(accepted, (1..=cut_off).collect::<Vec<_>>());
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

    #[test]
    fn malformed_emitter_output_is_an_error_not_a_panic() {
        assert_eq!(rendered_key_length(b"\"a\\n\"\n", "a\n").unwrap(), 5);
        assert_eq!(rendered_key_length("é\n".as_bytes(), "é").unwrap(), 1);
        for (bytes, reason) in [
            (b"\xff\n".as_slice(), "non-UTF-8"),
            (b"", "trailing LF"),
            (b"a", "trailing LF"),
        ] {
            let error = rendered_key_length(bytes, "k").unwrap_err();
            assert_eq!(error.code, "EMITTER_OUTPUT");
            assert_eq!(error.path, "k");
            assert!(error.message.contains(reason), "{}", error.message);
        }
    }

    #[test]
    fn multibyte_block_scalars_do_not_shift_later_positions() {
        // yaml-rust2 counts block-scalar content in UTF-8 bytes; every later
        // position must still resolve inside the input.
        for text in ["ﻷﻷﻷﻷ", "日本語のテキスト", "é", "🦀🦀"] {
            let yaml = format!("a: |\n  {text}\nb: cd\n");
            let jcs = convert(yaml.as_bytes(), Format::Kyaml, Output::Jcs).unwrap();
            assert_eq!(
                String::from_utf8(jcs).unwrap(),
                format!(r#"{{"a":"{text}\n","b":"cd"}}"#)
            );
        }
        // Positions after the scalar still locate later lexical errors.
        let error = convert(
            "a: |\n  ﻷﻷﻷﻷ\nb: c\n d\n".as_bytes(),
            Format::Kyaml,
            Output::Jcs,
        )
        .unwrap_err();
        assert_eq!((error.code, error.line), ("UNEXPECTED_INDENT", 4));
    }
}
