// SPDX-License-Identifier: AGPL-3.0-or-later
//! Format handlers for each supported format

pub mod plaintext;
pub mod markdown;
pub mod djot;
pub mod orgmode;

// FD-S01, FD-S02, FD-S03: SHOULD requirement implementations
pub mod asciidoc;
pub mod rst;
pub mod typst;

pub use plaintext::PlainTextHandler;
pub use markdown::MarkdownHandler;
pub use djot::DjotHandler;
pub use orgmode::OrgModeHandler;

// SHOULD handlers
pub use asciidoc::AsciidocHandler;
pub use rst::RstHandler;
pub use typst::TypstHandler;
