// SPDX-License-Identifier: PMPL-1.0-or-later
//! Format handlers for each supported format

pub mod djot;
pub mod markdown;
pub mod orgmode;
pub mod plaintext;

// FD-S01, FD-S02, FD-S03: SHOULD requirement implementations
pub mod asciidoc;
pub mod rst;
pub mod typst;

pub use djot::DjotHandler;
pub use markdown::MarkdownHandler;
pub use orgmode::OrgModeHandler;
pub use plaintext::PlainTextHandler;

// SHOULD handlers
pub use asciidoc::AsciidocHandler;
pub use rst::RstHandler;
pub use typst::TypstHandler;
