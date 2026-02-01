# Formatrix Docs - Claude Code Instructions

## Project Overview

Cross-platform document editor with format tabs, allowing users to view and edit the same document in multiple markup formats (TXT, MD, ADOC, DJOT, ORG, RST, TYP).

## Architecture

- **Core**: Rust library with unified AST for format conversion
- **GUI**: Tauri 2.0 with ReScript frontend (not TypeScript!)
- **TUI**: Ada with AdaCurses (matches git-hud pattern)
- **Storage**: ArangoDB for graph + document hybrid
- **Pipelines**: Nickel for import/export transformations

## Language Policy

### ALLOWED
- ReScript (UI components)
- Rust (core, GUI backend)
- Ada (TUI)
- Nickel (pipelines, config)
- Guile Scheme (SCM files)
- Deno (runtime)

### BANNED - Do Not Use
- TypeScript (use ReScript)
- Node.js/npm/bun (use Deno)
- Go (use Rust)
- Python (not applicable here)
- Makefiles (use justfile)

## Key Directories

```
crates/
├── formatrix-core/    # AST, parsers, renderers
├── formatrix-gui/     # Tauri commands
├── formatrix-db/      # ArangoDB client
└── formatrix-pipeline/ # Nickel executor

tui/src/               # Ada TUI source
ui/src/                # ReScript components
pipelines/             # Nickel pipeline definitions
container/             # Wolfi container configs
guix/                  # Guix channel + packages
nix/                   # Nix flake (fallback)
```

## Build Commands

```bash
just build           # Build all
just build-core      # Build Rust core only
just build-tui       # Build Ada TUI only
just build-ui        # Build ReScript UI only
just test            # Run all tests
just fmt             # Format all code
just lint            # Lint all code
```

## Checkpoint Files

Read at session start:
- `STATE.scm` - Project state and milestones
- `ECOSYSTEM.scm` - Position in ecosystem
- `META.scm` - Architecture decisions
- `PLAYBOOK.scm` - Operational procedures
- `AGENTIC.scm` - AI interaction patterns
- `NEUROSYM.scm` - Neurosymbolic config

## Format Conversion

The core library provides a unified AST:
- `Document` → contains metadata + blocks
- `Block` → paragraph, heading, list, code block, etc.
- `Inline` → text, emphasis, link, code span, etc.

Each format implements:
- `Parser` trait: raw content → AST
- `Renderer` trait: AST → raw content

## ReScript Conventions

- Use `@rescript/core` for stdlib
- TEA pattern: Model.res, Msg.res, Update logic in App.res
- Components in `src/components/`
- Bindings in `src/bindings/`

## Ada Conventions (TUI)

- Package specs in `.ads`, bodies in `.adb`
- Use `Terminal_Interface.Curses` for ncurses
- C FFI bindings to formatrix-core in `bindings/`
- Follow git-hud hybrid pattern for widgets

## Testing

```bash
just test-core       # Rust unit tests
just test-tui        # Ada compilation check
just test-ui         # ReScript tests
just test-integration # Full integration tests
```

## Container Usage

```bash
just container-build           # Build Wolfi image
just compose-up                # Start with ArangoDB
just container-run-tui         # Run TUI in container
```
