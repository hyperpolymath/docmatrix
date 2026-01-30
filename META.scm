;; SPDX-License-Identifier: MPL-2.0-or-later
;; META.scm - Project metadata and architectural decisions
;; Copyright (C) 2025 Jonathan D.A. Jewell

(define project-meta
  `((version . "1.0.0")

    (architecture-decisions
      ((adr-001
         (status . "accepted")
         (date . "2026-01-02")
         (title . "Unified AST for format conversion")
         (context . "Need to convert between 7+ document formats bidirectionally")
         (decision . "Create unified AST with Block/Inline/Document types that all formats map to/from")
         (consequences . ("Single representation for all formats"
                          "Lossy conversion for format-specific features"
                          "Clear trait boundaries for parsers/renderers")))

       (adr-002
         (status . "accepted")
         (date . "2026-01-02")
         (title . "Ada for TUI, Rust for core")
         (context . "Need safety-critical text handling and cross-platform performance")
         (decision . "Ada/AdaCurses for TUI (matches git-hud pattern), Rust for core library with C FFI")
         (consequences . ("Strong safety guarantees in TUI"
                          "Performance-critical paths in Rust"
                          "FFI boundary requires careful design")))

       (adr-003
         (status . "accepted")
         (date . "2026-01-02")
         (title . "Tauri 2.0 for GUI")
         (context . "Need desktop and mobile support with modern web UI")
         (decision . "Tauri 2.0 with ReScript frontend (not TypeScript)")
         (consequences . ("Single codebase for desktop/mobile"
                          "Rust backend with web frontend"
                          "Type-safe UI via ReScript")))

       (adr-004
         (status . "accepted")
         (date . "2026-01-02")
         (title . "ArangoDB for storage")
         (context . "Need hybrid graph + document storage for gist library and links")
         (decision . "ArangoDB with arangors Rust client")
         (consequences . ("Native graph queries for link visualization"
                          "Document storage for gists"
                          "Single database for both needs")))))

    (development-practices
      ((code-style . "rescript")
       (security . "openssf-scorecard")
       (testing . "property-based")
       (versioning . "semver")
       (documentation . "asciidoc")
       (branching . "trunk-based")))

    (design-rationale
      ((format-tabs . "Core UX - view same content in any format instantly")
       (graph-view . "Knowledge visualization like Anytype/Obsidian")
       (ada-tui . "Safety-critical text handling, matches git-hud pattern")
       (nickel-pipelines . "Declarative import/export transformations")))))
