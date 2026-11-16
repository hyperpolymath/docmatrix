// SPDX-License-Identifier: MPL-2.0
use docmatrix_data::{convert, Format, Output};
use proven_evidence::parser_contract::benchmark;

fn main() {
    // All frontends represent the same logical document and canonical bytes.
    let expected = br#"{"a":[1,2,3],"b":"document"}"#;
    for (name, input, format) in [
        ("kyaml", "b: document\na: [1, 2, 3]", Format::Kyaml),
        ("json", r#"{"b":"document","a":[1,2,3]}"#, Format::Json),
        (
            "dhall",
            "{ b = \"document\", a = [1, 2, 3] }",
            Format::Dhall,
        ),
    ] {
        assert_eq!(
            convert(input.as_bytes(), format, Output::Jcs).unwrap(),
            expected
        );
        let sample = benchmark(input.as_bytes(), 1000, |bytes| {
            convert(bytes, format, Output::Jcs).map_err(|error| error.to_string())
        })
        .expect("benchmark conversion must succeed");
        println!(
            "docmatrix_{name}_jcs iterations={} elapsed_ns={} input_bytes={} output_bytes={}",
            sample.iterations,
            sample.elapsed.as_nanos(),
            input.len(),
            expected.len()
        );
    }
}
