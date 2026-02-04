#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // TODO: Customize fuzzing logic for this repo
    // Example: parse input, test functions, etc.
    let _ = std::str::from_utf8(data);
});
