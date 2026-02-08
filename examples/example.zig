// SPDX-License-Identifier: AGPL-3.0-or-later
//! Example usage of zig-formatrix-ffi

const std = @import("std");
const formatrix = @import("formatrix");

pub fn main() !void {
    const allocator = std.heap.page_allocator;

    // Print library version
    std.debug.print("Formatrix version: {s}\n", .{formatrix.version()});

    // Parse some markdown
    const markdown_content: [:0]const u8 = "# Hello World\n\nThis is a paragraph.";

    var doc = try formatrix.Document.parse(markdown_content, .markdown);
    defer doc.deinit();

    std.debug.print("Parsed document with {d} blocks\n", .{doc.blockCount()});
    std.debug.print("Source format: {s}\n", .{doc.sourceFormat().label()});

    // Get title if present
    if (try doc.getTitle(allocator)) |title| {
        defer allocator.free(title);
        std.debug.print("Document title: {s}\n", .{title});
    }

    // Render to org-mode
    const org_output = try doc.render(.org_mode, allocator);
    defer allocator.free(org_output);

    std.debug.print("\nRendered to Org-mode:\n{s}\n", .{org_output});

    // Direct conversion helper
    const rst_output = try formatrix.convert(
        markdown_content,
        .markdown,
        .restructured_text,
        allocator,
    );
    defer allocator.free(rst_output);

    std.debug.print("\nDirect conversion to RST:\n{s}\n", .{rst_output});

    // Format detection
    const detected = formatrix.detectFormat("#+TITLE: Test\n* Heading");
    std.debug.print("\nDetected format for org content: {s}\n", .{detected.label()});
}
