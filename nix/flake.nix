# SPDX-License-Identifier: PMPL-1.0-or-later
# Formatrix Docs - Nix Flake (Fallback for non-Guix systems)
# Copyright (C) 2025 Jonathan D.A. Jewell
{
  description = "Cross-platform document editor with format tabs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust
            rustToolchain
            pkg-config
            openssl

            # Ada/GNAT
            gnat
            gprbuild

            # TUI dependencies
            ncurses

            # GUI dependencies (Tauri)
            gtk3
            webkitgtk
            libsoup

            # ReScript/Deno
            deno
            nodejs_20

            # External tools
            tesseract
            espeak-ng
            hunspell
            pandoc

            # Database
            arangodb

            # Nickel
            nickel

            # Development
            just
          ];

          shellHook = ''
            echo "Formatrix Docs development environment"
            echo "  Rust: $(rustc --version)"
            echo "  GNAT: $(gnat --version | head -1)"
            echo "  Deno: $(deno --version | head -1)"
            echo ""
            echo "Run 'just' to see available commands"
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        packages = {
          formatrix-core = pkgs.rustPlatform.buildRustPackage {
            pname = "formatrix-core";
            version = "0.1.0";
            src = ../.;
            cargoLock.lockFile = ../Cargo.lock;
            buildAndTestSubdir = "crates/formatrix-core";
          };

          formatrix-gui = pkgs.rustPlatform.buildRustPackage {
            pname = "formatrix-gui";
            version = "0.1.0";
            src = ../.;
            cargoLock.lockFile = ../Cargo.lock;
            buildAndTestSubdir = "crates/formatrix-gui";

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              gtk3
              webkitgtk
              libsoup
              openssl
            ];
          };

          formatrix-tui = pkgs.stdenv.mkDerivation {
            pname = "formatrix-tui";
            version = "0.1.0";
            src = ../tui;

            nativeBuildInputs = with pkgs; [
              gnat
              gprbuild
            ];

            buildInputs = with pkgs; [
              ncurses
            ];

            buildPhase = ''
              gprbuild -P formatrix_tui.gpr -XMODE=release
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp bin/formatrix-tui $out/bin/
            '';
          };

          default = self.packages.${system}.formatrix-gui;
        };
      }
    );
}
