#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Generic fuzzing for memory safety and crash detection
    if data.is_empty() || data.len() > 100000 {
        return;
    }

    // Test UTF-8 validity
    if let Ok(text) = std::str::from_utf8(data) {
        // Test string operations
        let _ = text.to_lowercase();
        let _ = text.chars().count();
        let _ = text.split_whitespace().collect::<Vec<_>>();
    }

    // Test binary data handling
    if data.len() >= 8 {
        let _chunk = &data[..8];
    }
});
