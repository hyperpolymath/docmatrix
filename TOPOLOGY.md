<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-19 -->

# DocMatrix — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              USER INTERFACE             │
                        │        (GUI / TUI / CLI)                │
                        └──────────┬───────────────────┬──────────┘
                                   │                   │
                                   ▼                   ▼
                        ┌───────────────────┐  ┌───────────────────┐
                        │   GUI (TAURI 2.0) │  │   TUI (ADA)       │
                        │ (ReScript / React)│  │ (AdaCurses)       │
                        └──────────┬────────┘  └──────────┬────────┘
                                   │                      │
                                   └──────────┬───────────┘
                                              │
                                              ▼
                        ┌─────────────────────────────────────────┐
                        │           RUST CORE (CRATES)            │
                        │                                         │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │  AST /    │  │  Nickel           │  │
                        │  │  Parsers  │  │  Pipelines        │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        │        │                 │              │
                        │  ┌─────▼─────┐  ┌────────▼──────────┐  │
                        │  │ ArangoDB  │  │  OCR / TTS        │  │
                        │  │ Client    │  │  Services         │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        └────────│─────────────────│──────────────┘
                                 │                 │
                                 ▼                 ▼
                        ┌─────────────────────────────────────────┐
                        │             DATA LAYER                  │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │ ArangoDB  │  │ Filesystem        │  │
                        │  │ (Graph)   │  │ (MD/ADOC/ORG/etc) │  │
                        │  └───────────┘  └───────────────────┘  │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  Justfile           .machine_readable/  │
                        │  Wolfi Containers   RSR Tier 2          │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
CORE ENGINE (RUST)
  Unified AST                       ██████████ 100%    Lossless conversion stable
  Format Parsers (MD/ADOC/etc)      ████████░░  80%    Typst parser refining
  Nickel Pipeline Executor          ██████████ 100%    Transformations active
  ArangoDB Client                   ██████████ 100%    Document persistence stable

USER INTERFACES
  Tauri 2.0 GUI                     ████████░░  80%    ReScript UI components active
  Ada TUI (AdaCurses)               ██████░░░░  60%    Layout logic in progress
  Graph Visualization               ████░░░░░░  40%    Initial D3.js prototyping

SERVICES & ACCESSIBILITY
  OCR Integration                   ██████░░░░  60%    Tesseract bindings active
  TTS / STT Support                 ████░░░░░░  40%    Initial engine integration
  Nickel Import/Export              ██████████ 100%    Standard pipelines verified

REPO INFRASTRUCTURE
  Justfile                          ██████████ 100%    Full multi-language build
  .machine_readable/                ██████████ 100%    STATE.a2ml tracking
  Wolfi Containers                  ██████████ 100%    Reproducible build env

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            ████████░░  ~80%   Core engine stable, UI refining
```

## Key Dependencies

```
Nickel Pipeline ───► Rust Parser ───► Unified AST ───► Renderers
                        │                 │                │
                        ▼                 ▼                ▼
                  ArangoDB ────────► Tauri / Ada ─────► User UI
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
