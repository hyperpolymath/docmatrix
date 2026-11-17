// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let Err(violation) = docmatrix_fuzz::check(data, docmatrix_data::Format::Dhall) {
        // libFuzzer records a finding only when the target terminates.
        eprintln!("{violation}");
        std::process::abort();
    }
});
