// SPDX-License-Identifier: AGPL-3.0-or-later
//! Build configuration for formatrix Zig bindings
//!
//! Links against libformatrix_core from the Rust crate.

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Create the formatrix module
    const formatrix_mod = b.addModule("formatrix", .{
        .root_source_file = b.path("formatrix.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Library artifact for linking
    const lib = b.addStaticLibrary(.{
        .name = "formatrix-zig",
        .root_source_file = b.path("formatrix.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Link against the Rust library
    lib.addLibraryPath(.{ .cwd_relative = "../../target/release" });
    lib.linkSystemLibrary("formatrix_core");
    lib.linkLibC();

    b.installArtifact(lib);

    // Example executable
    const exe = b.addExecutable(.{
        .name = "formatrix-example",
        .root_source_file = b.path("example.zig"),
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.addImport("formatrix", formatrix_mod);
    exe.addLibraryPath(.{ .cwd_relative = "../../target/release" });
    exe.linkSystemLibrary("formatrix_core");
    exe.linkLibC();

    b.installArtifact(exe);

    // Run step for example
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    const run_step = b.step("run", "Run the example");
    run_step.dependOn(&run_cmd.step);

    // Tests
    const tests = b.addTest(.{
        .root_source_file = b.path("formatrix.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_tests.step);
}
