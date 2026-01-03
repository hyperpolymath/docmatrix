// SPDX-License-Identifier: AGPL-3.0-or-later
//! Formatrix Core - Unified document AST and format converters
//!
//! This crate provides:
//! - A unified AST that all document formats convert to/from
//! - Parser and renderer traits for format handlers
//! - Implementations for 7 formats: TXT, MD, ADOC, DJOT, ORG, RST, TYP
//! - C FFI exports for the Ada TUI (FD-M10)

pub mod ast;
pub mod file_ops;
pub mod formats;
pub mod traits;

// FD-M10: C FFI exports for Ada TUI
#[cfg(feature = "ffi")]
pub mod ffi;

pub use ast::{Block, Document, DocumentMeta, Inline, SourceFormat};
pub use file_ops::{
    convert_file, convert_file_with_config, extension_for_format, format_from_content,
    format_from_extension, is_supported_extension, open_file, open_file_as,
    open_file_with_config, save_file, save_file_as, save_file_with_config, supported_extensions,
    FileError, FileInfo, FileResult, OpenedDocument,
};
pub use traits::{ConversionError, ParseConfig, Parser, RenderConfig, Renderer, Result};

// Re-export FFI types when enabled
#[cfg(feature = "ffi")]
pub use ffi::{
    formatrix_block_count, formatrix_convert, formatrix_detect_format, formatrix_free_document,
    formatrix_free_string, formatrix_get_format, formatrix_get_title, formatrix_parse,
    formatrix_render, formatrix_version, DocumentHandle, FfiFormat, FfiResult,
};
