// SPDX-License-Identifier: AGPL-3.0-or-later
//! C FFI exports for Ada TUI integration (FD-M10)
//!
//! These functions provide a C-compatible interface for the Ada TUI
//! to call into the Rust formatting core.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::ast::{Document, SourceFormat};
use crate::traits::{ParseConfig, Parser, RenderConfig, Renderer};

/// Opaque handle to a document
pub struct DocumentHandle {
    doc: Document,
}

/// Result code for FFI operations
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfiResult {
    Success = 0,
    InvalidInput = 1,
    ParseError = 2,
    RenderError = 3,
    UnsupportedFormat = 4,
    NullPointer = 5,
    Utf8Error = 6,
}

/// Document format for FFI
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfiFormat {
    PlainText = 0,
    Markdown = 1,
    AsciiDoc = 2,
    Djot = 3,
    OrgMode = 4,
    ReStructuredText = 5,
    Typst = 6,
}

impl From<FfiFormat> for SourceFormat {
    fn from(f: FfiFormat) -> Self {
        match f {
            FfiFormat::PlainText => SourceFormat::PlainText,
            FfiFormat::Markdown => SourceFormat::Markdown,
            FfiFormat::AsciiDoc => SourceFormat::AsciiDoc,
            FfiFormat::Djot => SourceFormat::Djot,
            FfiFormat::OrgMode => SourceFormat::OrgMode,
            FfiFormat::ReStructuredText => SourceFormat::ReStructuredText,
            FfiFormat::Typst => SourceFormat::Typst,
        }
    }
}

impl From<SourceFormat> for FfiFormat {
    fn from(f: SourceFormat) -> Self {
        match f {
            SourceFormat::PlainText => FfiFormat::PlainText,
            SourceFormat::Markdown => FfiFormat::Markdown,
            SourceFormat::AsciiDoc => FfiFormat::AsciiDoc,
            SourceFormat::Djot => FfiFormat::Djot,
            SourceFormat::OrgMode => FfiFormat::OrgMode,
            SourceFormat::ReStructuredText => FfiFormat::ReStructuredText,
            SourceFormat::Typst => FfiFormat::Typst,
        }
    }
}

/// Parse content into a document handle
///
/// # Safety
/// - `content` must be a valid null-terminated UTF-8 string
/// - `out_handle` must be a valid pointer to store the result
#[no_mangle]
pub unsafe extern "C" fn formatrix_parse(
    content: *const c_char,
    format: FfiFormat,
    out_handle: *mut *mut DocumentHandle,
) -> FfiResult {
    if content.is_null() || out_handle.is_null() {
        return FfiResult::NullPointer;
    }

    let content_str = match CStr::from_ptr(content).to_str() {
        Ok(s) => s,
        Err(_) => return FfiResult::Utf8Error,
    };

    let config = ParseConfig::default();
    let source_format: SourceFormat = format.into();

    let doc = match source_format {
        SourceFormat::PlainText => {
            use crate::formats::PlainTextHandler;
            match PlainTextHandler::new().parse(content_str, &config) {
                Ok(d) => d,
                Err(_) => return FfiResult::ParseError,
            }
        }
        SourceFormat::Markdown => {
            use crate::formats::MarkdownHandler;
            match MarkdownHandler::new().parse(content_str, &config) {
                Ok(d) => d,
                Err(_) => return FfiResult::ParseError,
            }
        }
        SourceFormat::Djot => {
            use crate::formats::DjotHandler;
            match DjotHandler::new().parse(content_str, &config) {
                Ok(d) => d,
                Err(_) => return FfiResult::ParseError,
            }
        }
        SourceFormat::OrgMode => {
            use crate::formats::OrgModeHandler;
            match OrgModeHandler::new().parse(content_str, &config) {
                Ok(d) => d,
                Err(_) => return FfiResult::ParseError,
            }
        }
        // FD-S01: AsciiDoc support
        SourceFormat::AsciiDoc => {
            use crate::formats::AsciidocHandler;
            match AsciidocHandler::new().parse(content_str, &config) {
                Ok(d) => d,
                Err(_) => return FfiResult::ParseError,
            }
        }
        // FD-S02: RST support
        SourceFormat::ReStructuredText => {
            use crate::formats::RstHandler;
            match RstHandler::new().parse(content_str, &config) {
                Ok(d) => d,
                Err(_) => return FfiResult::ParseError,
            }
        }
        // FD-S03: Typst support
        SourceFormat::Typst => {
            use crate::formats::TypstHandler;
            match TypstHandler::new().parse(content_str, &config) {
                Ok(d) => d,
                Err(_) => return FfiResult::ParseError,
            }
        }
    };

    let handle = Box::new(DocumentHandle { doc });
    *out_handle = Box::into_raw(handle);

    FfiResult::Success
}

/// Render a document to a string in the specified format
///
/// # Safety
/// - `handle` must be a valid document handle from `formatrix_parse`
/// - `out_content` must be a valid pointer to store the result
/// - `out_length` must be a valid pointer to store the length
#[no_mangle]
pub unsafe extern "C" fn formatrix_render(
    handle: *const DocumentHandle,
    format: FfiFormat,
    out_content: *mut *mut c_char,
    out_length: *mut usize,
) -> FfiResult {
    if handle.is_null() || out_content.is_null() || out_length.is_null() {
        return FfiResult::NullPointer;
    }

    let doc = &(*handle).doc;
    let config = RenderConfig::default();
    let target_format: SourceFormat = format.into();

    let output = match target_format {
        SourceFormat::PlainText => {
            use crate::formats::PlainTextHandler;
            match PlainTextHandler::new().render(doc, &config) {
                Ok(s) => s,
                Err(_) => return FfiResult::RenderError,
            }
        }
        SourceFormat::Markdown => {
            use crate::formats::MarkdownHandler;
            match MarkdownHandler::new().render(doc, &config) {
                Ok(s) => s,
                Err(_) => return FfiResult::RenderError,
            }
        }
        SourceFormat::Djot => {
            use crate::formats::DjotHandler;
            match DjotHandler::new().render(doc, &config) {
                Ok(s) => s,
                Err(_) => return FfiResult::RenderError,
            }
        }
        SourceFormat::OrgMode => {
            use crate::formats::OrgModeHandler;
            match OrgModeHandler::new().render(doc, &config) {
                Ok(s) => s,
                Err(_) => return FfiResult::RenderError,
            }
        }
        // FD-S01: AsciiDoc support
        SourceFormat::AsciiDoc => {
            use crate::formats::AsciidocHandler;
            match AsciidocHandler::new().render(doc, &config) {
                Ok(s) => s,
                Err(_) => return FfiResult::RenderError,
            }
        }
        // FD-S02: RST support
        SourceFormat::ReStructuredText => {
            use crate::formats::RstHandler;
            match RstHandler::new().render(doc, &config) {
                Ok(s) => s,
                Err(_) => return FfiResult::RenderError,
            }
        }
        // FD-S03: Typst support
        SourceFormat::Typst => {
            use crate::formats::TypstHandler;
            match TypstHandler::new().render(doc, &config) {
                Ok(s) => s,
                Err(_) => return FfiResult::RenderError,
            }
        }
    };

    let c_string = match CString::new(output.clone()) {
        Ok(s) => s,
        Err(_) => return FfiResult::InvalidInput,
    };

    *out_length = output.len();
    *out_content = c_string.into_raw();

    FfiResult::Success
}

/// Get the title of a document
///
/// # Safety
/// - `handle` must be a valid document handle
/// - `out_title` must be a valid pointer
/// - `out_length` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn formatrix_get_title(
    handle: *const DocumentHandle,
    out_title: *mut *mut c_char,
    out_length: *mut usize,
) -> FfiResult {
    if handle.is_null() || out_title.is_null() || out_length.is_null() {
        return FfiResult::NullPointer;
    }

    let doc = &(*handle).doc;
    let title = doc.meta.title.clone().unwrap_or_default();

    let c_string = match CString::new(title.clone()) {
        Ok(s) => s,
        Err(_) => return FfiResult::InvalidInput,
    };

    *out_length = title.len();
    *out_title = c_string.into_raw();

    FfiResult::Success
}

/// Get the number of blocks in a document
///
/// # Safety
/// - `handle` must be a valid document handle
#[no_mangle]
pub unsafe extern "C" fn formatrix_block_count(handle: *const DocumentHandle) -> usize {
    if handle.is_null() {
        return 0;
    }
    (*handle).doc.content.len()
}

/// Get the source format of a document
///
/// # Safety
/// - `handle` must be a valid document handle
#[no_mangle]
pub unsafe extern "C" fn formatrix_get_format(handle: *const DocumentHandle) -> FfiFormat {
    if handle.is_null() {
        return FfiFormat::PlainText;
    }
    (*handle).doc.source_format.into()
}

/// Free a document handle
///
/// # Safety
/// - `handle` must be a valid document handle or null
#[no_mangle]
pub unsafe extern "C" fn formatrix_free_document(handle: *mut DocumentHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Free a string allocated by the library
///
/// # Safety
/// - `s` must be a valid string from this library or null
#[no_mangle]
pub unsafe extern "C" fn formatrix_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Get library version
///
/// # Safety
/// Returns a static string, do not free
#[no_mangle]
pub extern "C" fn formatrix_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

/// Detect format from content
///
/// # Safety
/// - `content` must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn formatrix_detect_format(content: *const c_char) -> FfiFormat {
    if content.is_null() {
        return FfiFormat::PlainText;
    }

    let content_str = match CStr::from_ptr(content).to_str() {
        Ok(s) => s,
        Err(_) => return FfiFormat::PlainText,
    };

    let trimmed = content_str.trim();

    // Check for org-mode markers
    if trimmed.starts_with("#+") || trimmed.contains("\n#+") {
        return FfiFormat::OrgMode;
    }

    // Check for AsciiDoc markers
    if trimmed.starts_with("= ") || trimmed.starts_with(":toc:") {
        return FfiFormat::AsciiDoc;
    }

    // Check for Markdown markers
    if trimmed.starts_with("# ") || trimmed.contains("```") {
        return FfiFormat::Markdown;
    }

    // Check for Djot markers
    if trimmed.contains("{.") || trimmed.contains("[^") {
        return FfiFormat::Djot;
    }

    // Check for RST markers
    if trimmed.contains(".. ") && trimmed.contains("::") {
        return FfiFormat::ReStructuredText;
    }

    // Check for Typst markers
    if trimmed.contains("#let") || trimmed.contains("#{") {
        return FfiFormat::Typst;
    }

    FfiFormat::PlainText
}

/// Convert content from one format to another
///
/// # Safety
/// - All pointers must be valid
#[no_mangle]
pub unsafe extern "C" fn formatrix_convert(
    content: *const c_char,
    from_format: FfiFormat,
    to_format: FfiFormat,
    out_content: *mut *mut c_char,
    out_length: *mut usize,
) -> FfiResult {
    if content.is_null() || out_content.is_null() || out_length.is_null() {
        return FfiResult::NullPointer;
    }

    // Parse input
    let mut handle: *mut DocumentHandle = ptr::null_mut();
    let parse_result = formatrix_parse(content, from_format, &mut handle);
    if parse_result != FfiResult::Success {
        return parse_result;
    }

    // Render output
    let render_result = formatrix_render(handle, to_format, out_content, out_length);

    // Free the handle
    formatrix_free_document(handle);

    render_result
}

/// Open a file and parse it into a document handle
///
/// # Safety
/// - `path` must be a valid null-terminated UTF-8 file path
/// - `out_handle` must be a valid pointer to store the result
/// - `out_format` must be a valid pointer to store the detected format
#[no_mangle]
pub unsafe extern "C" fn formatrix_open_file(
    path: *const c_char,
    out_handle: *mut *mut DocumentHandle,
    out_format: *mut FfiFormat,
) -> FfiResult {
    if path.is_null() || out_handle.is_null() || out_format.is_null() {
        return FfiResult::NullPointer;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return FfiResult::Utf8Error,
    };

    use crate::file_ops;
    match file_ops::open_file(path_str) {
        Ok(opened) => {
            *out_format = opened.file_info.format.into();
            let handle = Box::new(DocumentHandle { doc: opened.document });
            *out_handle = Box::into_raw(handle);
            FfiResult::Success
        }
        Err(e) => match e {
            file_ops::FileError::Io(_) => FfiResult::InvalidInput,
            file_ops::FileError::Parse(_) => FfiResult::ParseError,
            file_ops::FileError::UnknownFormat { .. } => FfiResult::UnsupportedFormat,
            file_ops::FileError::UnsupportedFormat { .. } => FfiResult::UnsupportedFormat,
            file_ops::FileError::Render(_) => FfiResult::RenderError,
        },
    }
}

/// Save a document to a file
///
/// Format is determined by the file extension. If no extension matches,
/// uses the document's source format.
///
/// # Safety
/// - `handle` must be a valid document handle
/// - `path` must be a valid null-terminated UTF-8 file path
#[no_mangle]
pub unsafe extern "C" fn formatrix_save_file(
    handle: *const DocumentHandle,
    path: *const c_char,
) -> FfiResult {
    if handle.is_null() || path.is_null() {
        return FfiResult::NullPointer;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return FfiResult::Utf8Error,
    };

    let doc = &(*handle).doc;

    use crate::file_ops;
    match file_ops::save_file(doc, path_str) {
        Ok(()) => FfiResult::Success,
        Err(e) => match e {
            file_ops::FileError::Io(_) => FfiResult::InvalidInput,
            file_ops::FileError::Render(_) => FfiResult::RenderError,
            file_ops::FileError::UnsupportedFormat { .. } => FfiResult::UnsupportedFormat,
            _ => FfiResult::RenderError,
        },
    }
}

/// Save a document to a file in a specific format
///
/// # Safety
/// - `handle` must be a valid document handle
/// - `path` must be a valid null-terminated UTF-8 file path
#[no_mangle]
pub unsafe extern "C" fn formatrix_save_file_as(
    handle: *const DocumentHandle,
    path: *const c_char,
    format: FfiFormat,
) -> FfiResult {
    if handle.is_null() || path.is_null() {
        return FfiResult::NullPointer;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return FfiResult::Utf8Error,
    };

    let doc = &(*handle).doc;
    let target_format: SourceFormat = format.into();

    use crate::file_ops;
    use crate::traits::RenderConfig;
    match file_ops::save_file_as(doc, path_str, target_format, &RenderConfig::default()) {
        Ok(()) => FfiResult::Success,
        Err(e) => match e {
            file_ops::FileError::Io(_) => FfiResult::InvalidInput,
            file_ops::FileError::Render(_) => FfiResult::RenderError,
            file_ops::FileError::UnsupportedFormat { .. } => FfiResult::UnsupportedFormat,
            _ => FfiResult::RenderError,
        },
    }
}

/// Detect format from file path (by extension)
///
/// # Safety
/// - `path` must be a valid null-terminated UTF-8 file path
#[no_mangle]
pub unsafe extern "C" fn formatrix_detect_file_format(path: *const c_char) -> FfiFormat {
    if path.is_null() {
        return FfiFormat::PlainText;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return FfiFormat::PlainText,
    };

    use crate::file_ops;
    use std::path::Path;

    match file_ops::format_from_extension(Path::new(path_str)) {
        Some(format) => format.into(),
        None => FfiFormat::PlainText,
    }
}

/// Get the file extension for a format
///
/// # Safety
/// Returns a static string, do not free
#[no_mangle]
pub extern "C" fn formatrix_format_extension(format: FfiFormat) -> *const c_char {
    static EXT_TXT: &[u8] = b"txt\0";
    static EXT_MD: &[u8] = b"md\0";
    static EXT_ADOC: &[u8] = b"adoc\0";
    static EXT_DJ: &[u8] = b"dj\0";
    static EXT_ORG: &[u8] = b"org\0";
    static EXT_RST: &[u8] = b"rst\0";
    static EXT_TYP: &[u8] = b"typ\0";

    let ptr = match format {
        FfiFormat::PlainText => EXT_TXT.as_ptr(),
        FfiFormat::Markdown => EXT_MD.as_ptr(),
        FfiFormat::AsciiDoc => EXT_ADOC.as_ptr(),
        FfiFormat::Djot => EXT_DJ.as_ptr(),
        FfiFormat::OrgMode => EXT_ORG.as_ptr(),
        FfiFormat::ReStructuredText => EXT_RST.as_ptr(),
        FfiFormat::Typst => EXT_TYP.as_ptr(),
    };
    ptr as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_parse_and_render() {
        let content = CString::new("# Hello\n\nWorld").unwrap();
        let mut handle: *mut DocumentHandle = ptr::null_mut();

        unsafe {
            let result = formatrix_parse(content.as_ptr(), FfiFormat::Markdown, &mut handle);
            assert_eq!(result, FfiResult::Success);
            assert!(!handle.is_null());

            let count = formatrix_block_count(handle);
            assert!(count > 0);

            formatrix_free_document(handle);
        }
    }

    #[test]
    fn test_detect_format() {
        let md = CString::new("# Heading\n\nContent").unwrap();
        let org = CString::new("#+TITLE: Test\n* Heading").unwrap();

        unsafe {
            assert_eq!(formatrix_detect_format(md.as_ptr()), FfiFormat::Markdown);
            assert_eq!(formatrix_detect_format(org.as_ptr()), FfiFormat::OrgMode);
        }
    }
}
