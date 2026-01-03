// SPDX-License-Identifier: AGPL-3.0-or-later
//! Zig bindings for formatrix-core
//!
//! Provides a type-safe Zig interface to the Formatrix document library.
//! Links against libformatrix_core.so/dylib/dll

const std = @import("std");

/// Document format types
pub const Format = enum(c_int) {
    plain_text = 0,
    markdown = 1,
    asciidoc = 2,
    djot = 3,
    org_mode = 4,
    restructured_text = 5,
    typst = 6,

    /// Get file extension for this format
    pub fn extension(self: Format) [:0]const u8 {
        return switch (self) {
            .plain_text => "txt",
            .markdown => "md",
            .asciidoc => "adoc",
            .djot => "dj",
            .org_mode => "org",
            .restructured_text => "rst",
            .typst => "typ",
        };
    }

    /// Get display label for this format
    pub fn label(self: Format) [:0]const u8 {
        return switch (self) {
            .plain_text => "TXT",
            .markdown => "MD",
            .asciidoc => "ADOC",
            .djot => "DJOT",
            .org_mode => "ORG",
            .restructured_text => "RST",
            .typst => "TYP",
        };
    }
};

/// Result codes from FFI operations
pub const Result = enum(c_int) {
    success = 0,
    invalid_input = 1,
    parse_error = 2,
    render_error = 3,
    unsupported_format = 4,
    null_pointer = 5,
    utf8_error = 6,

    pub fn isSuccess(self: Result) bool {
        return self == .success;
    }

    pub fn toError(self: Result) ?Error {
        return switch (self) {
            .success => null,
            .invalid_input => Error.InvalidInput,
            .parse_error => Error.ParseError,
            .render_error => Error.RenderError,
            .unsupported_format => Error.UnsupportedFormat,
            .null_pointer => Error.NullPointer,
            .utf8_error => Error.Utf8Error,
        };
    }
};

/// Errors that can occur during formatrix operations
pub const Error = error{
    InvalidInput,
    ParseError,
    RenderError,
    UnsupportedFormat,
    NullPointer,
    Utf8Error,
};

/// Opaque document handle
pub const DocumentHandle = opaque {};

// External C functions from libformatrix_core
extern "c" fn formatrix_parse(
    content: [*:0]const u8,
    format: Format,
    out_handle: *?*DocumentHandle,
) Result;

extern "c" fn formatrix_render(
    handle: *const DocumentHandle,
    format: Format,
    out_content: *?[*:0]u8,
    out_length: *usize,
) Result;

extern "c" fn formatrix_open_file(
    path: [*:0]const u8,
    out_handle: *?*DocumentHandle,
    out_format: *Format,
) Result;

extern "c" fn formatrix_save_file(
    handle: *const DocumentHandle,
    path: [*:0]const u8,
) Result;

extern "c" fn formatrix_save_file_as(
    handle: *const DocumentHandle,
    path: [*:0]const u8,
    format: Format,
) Result;

extern "c" fn formatrix_get_title(
    handle: *const DocumentHandle,
    out_title: *?[*:0]u8,
    out_length: *usize,
) Result;

extern "c" fn formatrix_block_count(handle: *const DocumentHandle) usize;

extern "c" fn formatrix_get_format(handle: *const DocumentHandle) Format;

extern "c" fn formatrix_detect_format(content: [*:0]const u8) Format;

extern "c" fn formatrix_detect_file_format(path: [*:0]const u8) Format;

extern "c" fn formatrix_convert(
    content: [*:0]const u8,
    from_format: Format,
    to_format: Format,
    out_content: *?[*:0]u8,
    out_length: *usize,
) Result;

extern "c" fn formatrix_free_document(handle: ?*DocumentHandle) void;

extern "c" fn formatrix_free_string(s: ?[*:0]u8) void;

extern "c" fn formatrix_version() [*:0]const u8;

/// A parsed document with automatic resource management
pub const Document = struct {
    handle: *DocumentHandle,

    const Self = @This();

    /// Parse content in the specified format
    pub fn parse(content: [:0]const u8, format: Format) Error!Self {
        var handle: ?*DocumentHandle = null;
        const result = formatrix_parse(content.ptr, format, &handle);

        if (result.toError()) |err| {
            return err;
        }

        return Self{ .handle = handle.? };
    }

    /// Open a file and parse it
    pub fn openFile(path: [:0]const u8) Error!struct { doc: Self, format: Format } {
        var handle: ?*DocumentHandle = null;
        var format: Format = .plain_text;
        const result = formatrix_open_file(path.ptr, &handle, &format);

        if (result.toError()) |err| {
            return err;
        }

        return .{
            .doc = Self{ .handle = handle.? },
            .format = format,
        };
    }

    /// Free the document resources
    pub fn deinit(self: *Self) void {
        formatrix_free_document(self.handle);
        self.handle = undefined;
    }

    /// Render the document to the specified format
    pub fn render(self: Self, format: Format, allocator: std.mem.Allocator) Error![]u8 {
        var content: ?[*:0]u8 = null;
        var length: usize = 0;

        const result = formatrix_render(self.handle, format, &content, &length);

        if (result.toError()) |err| {
            return err;
        }

        defer formatrix_free_string(content);

        // Copy to Zig-managed memory
        const owned = try allocator.alloc(u8, length);
        @memcpy(owned, content.?[0..length]);
        return owned;
    }

    /// Save the document to a file (format detected from extension)
    pub fn saveFile(self: Self, path: [:0]const u8) Error!void {
        const result = formatrix_save_file(self.handle, path.ptr);
        if (result.toError()) |err| {
            return err;
        }
    }

    /// Save the document to a file in a specific format
    pub fn saveFileAs(self: Self, path: [:0]const u8, format: Format) Error!void {
        const result = formatrix_save_file_as(self.handle, path.ptr, format);
        if (result.toError()) |err| {
            return err;
        }
    }

    /// Get the document title (if any)
    pub fn getTitle(self: Self, allocator: std.mem.Allocator) Error!?[]u8 {
        var title: ?[*:0]u8 = null;
        var length: usize = 0;

        const result = formatrix_get_title(self.handle, &title, &length);

        if (result.toError()) |err| {
            return err;
        }

        if (length == 0) {
            return null;
        }

        defer formatrix_free_string(title);

        const owned = try allocator.alloc(u8, length);
        @memcpy(owned, title.?[0..length]);
        return owned;
    }

    /// Get the number of blocks in the document
    pub fn blockCount(self: Self) usize {
        return formatrix_block_count(self.handle);
    }

    /// Get the source format of the document
    pub fn sourceFormat(self: Self) Format {
        return formatrix_get_format(self.handle);
    }
};

/// Convert content between formats
pub fn convert(
    content: [:0]const u8,
    from_format: Format,
    to_format: Format,
    allocator: std.mem.Allocator,
) Error![]u8 {
    var out_content: ?[*:0]u8 = null;
    var out_length: usize = 0;

    const result = formatrix_convert(
        content.ptr,
        from_format,
        to_format,
        &out_content,
        &out_length,
    );

    if (result.toError()) |err| {
        return err;
    }

    defer formatrix_free_string(out_content);

    const owned = try allocator.alloc(u8, out_length);
    @memcpy(owned, out_content.?[0..out_length]);
    return owned;
}

/// Detect format from content using heuristics
pub fn detectFormat(content: [:0]const u8) Format {
    return formatrix_detect_format(content.ptr);
}

/// Detect format from file path (by extension)
pub fn detectFileFormat(path: [:0]const u8) Format {
    return formatrix_detect_file_format(path.ptr);
}

/// Get the library version
pub fn version() [:0]const u8 {
    return std.mem.span(formatrix_version());
}

// Tests
test "format extension" {
    try std.testing.expectEqualStrings("md", Format.markdown.extension());
    try std.testing.expectEqualStrings("org", Format.org_mode.extension());
}

test "format label" {
    try std.testing.expectEqualStrings("MD", Format.markdown.label());
    try std.testing.expectEqualStrings("ORG", Format.org_mode.label());
}

test "result conversion" {
    try std.testing.expect(Result.success.isSuccess());
    try std.testing.expect(!Result.parse_error.isSuccess());
    try std.testing.expect(Result.success.toError() == null);
    try std.testing.expect(Result.parse_error.toError() == Error.ParseError);
}
