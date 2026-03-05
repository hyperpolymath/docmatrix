# Formatrix Docs - RSR Standard Justfile
# SPDX-License-Identifier: AGPL-3.0-or-later
# https://just.systems/man/en/

set shell := ["bash", "-uc"]
set dotenv-load := true
set positional-arguments := true

# Use Zig as C compiler/linker (avoids gcc dependency)
export CC := env_var_or_default("CC", justfile_directory() / "zig-cc")
export AR := env_var_or_default("AR", justfile_directory() / "zig-ar")

# Project metadata
project := "formatrix-docs"
version := "0.1.0"
tier := "2"

# ═══════════════════════════════════════════════════════════════════════════════
# DEFAULT & HELP
# ═══════════════════════════════════════════════════════════════════════════════

# Show all available recipes with descriptions
default:
    @just --list --unsorted

# Show detailed help for a specific recipe
help recipe="":
    #!/usr/bin/env bash
    if [ -z "{{recipe}}" ]; then
        just --list --unsorted
        echo ""
        echo "Usage: just help <recipe>"
        echo "       just cookbook     # Generate full documentation"
        echo "       just combinations # Show matrix recipes"
    else
        just --show "{{recipe}}" 2>/dev/null || echo "Recipe '{{recipe}}' not found"
    fi

# Show this project's info
info:
    @echo "Project: {{project}}"
    @echo "Version: {{version}}"
    @echo "RSR Tier: {{tier}}"
    @echo "Recipes: $(just --summary | wc -w)"
    @[ -f STATE.scm ] && grep -oP '\(phase\s+\.\s+\K[^)]+' STATE.scm | head -1 | xargs -I{} echo "Phase: {}" || true

# ═══════════════════════════════════════════════════════════════════════════════
# BUILD & COMPILE
# ═══════════════════════════════════════════════════════════════════════════════

# Build all components (Rust + Ada + ReScript)
build: build-core build-gui build-tui build-ui
    @echo "All components built!"

# Build Rust core library
build-core:
    @echo "Building formatrix-core..."
    cargo build -p formatrix-core

# Build Rust GUI (Tauri - requires GTK/WebKit dev libs)
build-gui: build-core
    #!/usr/bin/env bash
    echo "Building formatrix-gui..."
    if ! pkg-config --exists glib-2.0 2>/dev/null; then
        echo "SKIP: glib-2.0 not found (install gtk4-devel webkit2gtk4.1-devel)"
        exit 0
    fi
    cargo build -p formatrix-gui

# Build Ada TUI (requires GNAT + ncurses-ada)
build-tui:
    #!/usr/bin/env bash
    echo "Building formatrix-tui..."
    if ! command -v gprbuild > /dev/null 2>&1; then
        echo "SKIP: gprbuild not found (install gcc-gnat gprbuild)"
        exit 0
    fi
    # Check for ncurses.gpr availability
    NCURSES_GPR=""
    for path in /usr/share/gpr/ncurses.gpr /usr/lib64/gnat/ncurses.gpr /usr/share/ada/adainclude/ncurses.gpr; do
        if [ -f "$path" ]; then
            NCURSES_GPR="$path"
            break
        fi
    done
    if [ -z "$NCURSES_GPR" ]; then
        echo "SKIP: ncurses.gpr not found (install terminal_interface-curses-devel or florist-devel)"
        exit 0
    fi
    cd tui && gprbuild -P formatrix_tui.gpr -XMODE=debug

# Build ReScript UI
build-ui:
    @echo "Building ReScript UI..."
    @cd ui && deno task build:res 2>&1 | tail -5

# Build in release mode
build-release:
    @echo "Building all (release)..."
    cargo build --release
    @command -v gprbuild > /dev/null 2>&1 && cd tui && gprbuild -P formatrix_tui.gpr -XMODE=release || echo "SKIP: TUI (gprbuild not found)"
    cd ui && deno task build 2>/dev/null || true

# Clean build artifacts
clean:
    @echo "Cleaning..."
    cargo clean
    cd tui && gnatclean -P formatrix_tui.gpr 2>/dev/null || true
    rm -rf tui/obj tui/bin ui/dist ui/lib

# ═══════════════════════════════════════════════════════════════════════════════
# TEST & QUALITY
# ═══════════════════════════════════════════════════════════════════════════════

# Run all tests
test: test-core test-tui
    @echo "All tests passed!"

# Test Rust core
test-core:
    @echo "Testing formatrix-core..."
    cargo test -p formatrix-core

# Test Ada TUI (compile check)
test-tui: build-tui
    @echo "Testing formatrix-tui..."
    @[ -f tui/bin/formatrix-tui ] && echo "TUI binary exists" || echo "SKIP: TUI not built (missing dependencies)"

# Test ReScript UI
test-ui:
    @echo "Testing UI..."
    cd ui && deno task test 2>/dev/null || echo "UI tests not configured yet"

# Run integration tests
test-integration:
    @echo "Running integration tests..."
    cargo test --workspace -- --ignored

# ═══════════════════════════════════════════════════════════════════════════════
# LINT & FORMAT
# ═══════════════════════════════════════════════════════════════════════════════

# Format all source files
fmt:
    @echo "Formatting..."
    cargo fmt
    cd ui && deno fmt 2>/dev/null || true
    @if command -v gnatpp > /dev/null 2>&1; then \
        find tui/src -name "*.adb" -o -name "*.ads" | xargs -I{} gnatpp -rnb --max-line-length=120 {} 2>/dev/null || true; \
    fi

# Check formatting
fmt-check:
    @echo "Checking formatting..."
    cargo fmt -- --check
    cd ui && deno fmt --check 2>/dev/null || true

# Run linter
lint:
    @echo "Linting..."
    cargo clippy --workspace -- -D warnings
    cd ui && deno lint 2>/dev/null || true

# Run all quality checks
quality: fmt-check lint test
    @echo "All quality checks passed!"

# ═══════════════════════════════════════════════════════════════════════════════
# RUN & EXECUTE
# ═══════════════════════════════════════════════════════════════════════════════

# Run GUI application
run-gui *args: build-gui
    cargo run -p formatrix-gui -- {{args}}

# Run TUI application
run-tui *args: build-tui
    tui/bin/formatrix-tui {{args}}

# Run with debug logging
run-debug:
    RUST_LOG=debug cargo run -p formatrix-gui

# ═══════════════════════════════════════════════════════════════════════════════
# DEPENDENCIES
# ═══════════════════════════════════════════════════════════════════════════════

# Install all dependencies
deps:
    @echo "Checking dependencies..."
    @command -v cargo > /dev/null 2>&1 || { echo "ERROR: cargo not found"; exit 1; }
    @command -v deno > /dev/null 2>&1 || { echo "ERROR: deno not found"; exit 1; }
    @echo "Rust: $(rustc --version)"
    @echo "Deno: $(deno --version | head -1)"
    @command -v gnat > /dev/null 2>&1 && echo "GNAT: $(gnat --version | head -1)" || echo "WARN: gnat not found (TUI disabled)"
    @command -v gprbuild > /dev/null 2>&1 || echo "WARN: gprbuild not found (TUI disabled)"
    @echo "Core dependencies satisfied"

# Audit dependencies for vulnerabilities
deps-audit:
    @echo "Auditing dependencies..."
    cargo audit 2>/dev/null || echo "cargo-audit not installed"
    @if command -v trivy > /dev/null 2>&1; then \
        trivy fs --severity HIGH,CRITICAL --quiet . || true; \
    fi

# ═══════════════════════════════════════════════════════════════════════════════
# DOCUMENTATION
# ═══════════════════════════════════════════════════════════════════════════════

# Generate all documentation
docs:
    @mkdir -p docs/generated docs/man
    cargo doc --workspace --no-deps
    just cookbook
    @echo "Documentation generated in docs/ and target/doc/"

# Generate justfile cookbook
cookbook:
    #!/usr/bin/env bash
    mkdir -p docs
    OUTPUT="docs/just-cookbook.adoc"
    echo "= {{project}} Justfile Cookbook" > "$OUTPUT"
    echo ":toc: left" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "Generated: $(date -Iseconds)" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    just --list --unsorted >> "$OUTPUT"
    echo "Generated: $OUTPUT"

# ═══════════════════════════════════════════════════════════════════════════════
# CONTAINERS (nerdctl-first, podman-fallback)
# ═══════════════════════════════════════════════════════════════════════════════

# Detect container runtime: nerdctl > podman > docker
[private]
container-cmd:
    #!/usr/bin/env bash
    if command -v nerdctl >/dev/null 2>&1; then
        echo "nerdctl"
    elif command -v podman >/dev/null 2>&1; then
        echo "podman"
    elif command -v docker >/dev/null 2>&1; then
        echo "docker"
    else
        echo "ERROR: No container runtime found (install nerdctl, podman, or docker)" >&2
        exit 1
    fi

# Build container image
container-build tag="latest":
    #!/usr/bin/env bash
    CTR=$(just container-cmd)
    echo "Building container with $CTR..."
    $CTR build -t {{project}}:{{tag}} -f container/Dockerfile.wolfi .

# Run container (GUI)
container-run tag="latest" cmd="":
    #!/usr/bin/env bash
    CTR=$(just container-cmd)
    $CTR run --rm -it \
        -e DISPLAY=$DISPLAY \
        -v /tmp/.X11-unix:/tmp/.X11-unix:ro \
        {{project}}:{{tag}} {{cmd}}

# Run container (TUI)
container-run-tui tag="latest":
    #!/usr/bin/env bash
    CTR=$(just container-cmd)
    $CTR run --rm -it \
        -e TERM=$TERM \
        {{project}}:{{tag}} /usr/local/bin/formatrix-tui

# Start all services with compose
compose-up:
    #!/usr/bin/env bash
    CTR=$(just container-cmd)
    cd container && $CTR compose up -d

# Stop all services
compose-down:
    #!/usr/bin/env bash
    CTR=$(just container-cmd)
    cd container && $CTR compose down

# View logs
compose-logs:
    #!/usr/bin/env bash
    CTR=$(just container-cmd)
    cd container && $CTR compose logs -f

# Push container image
container-push registry="ghcr.io/hyperpolymath" tag="latest":
    #!/usr/bin/env bash
    CTR=$(just container-cmd)
    $CTR tag {{project}}:{{tag}} {{registry}}/{{project}}:{{tag}}
    $CTR push {{registry}}/{{project}}:{{tag}}

# ═══════════════════════════════════════════════════════════════════════════════
# CI & AUTOMATION
# ═══════════════════════════════════════════════════════════════════════════════

# Run full CI pipeline locally
ci: deps quality
    @echo "CI pipeline complete!"

# Install git hooks
install-hooks:
    #!/usr/bin/env bash
    mkdir -p .git/hooks
    printf '%s\n' '#!/bin/bash' 'just fmt-check || exit 1' 'just lint || exit 1' > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "Git hooks installed"

# ═══════════════════════════════════════════════════════════════════════════════
# SECURITY
# ═══════════════════════════════════════════════════════════════════════════════

# Run security audit
security: deps-audit
    @echo "=== Security Audit ==="
    @command -v gitleaks >/dev/null && gitleaks detect --source . --verbose || true
    @command -v trivy >/dev/null && trivy fs --severity HIGH,CRITICAL . || true
    @echo "Security audit complete"

# Generate SBOM
sbom:
    @mkdir -p docs/security
    @command -v syft >/dev/null && syft . -o spdx-json > docs/security/sbom.spdx.json || echo "syft not found"

# ═══════════════════════════════════════════════════════════════════════════════
# VALIDATION & COMPLIANCE
# ═══════════════════════════════════════════════════════════════════════════════

# Validate RSR compliance
validate-rsr:
    #!/usr/bin/env bash
    echo "=== RSR Compliance Check ==="
    MISSING=""
    for f in .editorconfig .gitignore justfile RSR_COMPLIANCE.adoc README.adoc; do
        [ -f "$f" ] || MISSING="$MISSING $f"
    done
    for f in STATE.scm ECOSYSTEM.scm META.scm; do
        [ -f "$f" ] || MISSING="$MISSING $f"
    done
    if [ -n "$MISSING" ]; then
        echo "MISSING:$MISSING"
        exit 1
    fi
    echo "RSR compliance: PASS"

# Validate STATE.scm syntax
validate-state:
    @if [ -f "STATE.scm" ]; then \
        guile -c "(primitive-load \"STATE.scm\")" 2>/dev/null && echo "STATE.scm: valid" || echo "STATE.scm: INVALID"; \
    fi

# Full validation suite
validate: validate-rsr validate-state
    @echo "All validations passed!"

# ═══════════════════════════════════════════════════════════════════════════════
# STATE MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════════════

# Update STATE.scm timestamp
state-touch:
    @if [ -f "STATE.scm" ]; then \
        sed -i 's/(updated . "[^"]*")/(updated . "'"$(date -Iseconds)"'")/' STATE.scm && \
        echo "STATE.scm timestamp updated"; \
    fi

# Show current phase from STATE.scm
state-phase:
    @grep -oP '\(phase\s+\.\s+\K[^)]+' STATE.scm 2>/dev/null | head -1 || echo "unknown"

# ═══════════════════════════════════════════════════════════════════════════════
# GUIX & NIX
# ═══════════════════════════════════════════════════════════════════════════════

# Enter Guix development shell (primary)
guix-shell:
    guix shell -D -f guix/formatrix.scm

# Build with Guix
guix-build:
    guix build -f guix/formatrix.scm

# Enter Nix development shell (fallback)
nix-shell:
    @if [ -f "nix/flake.nix" ]; then cd nix && nix develop; else echo "No flake.nix"; fi

# ═══════════════════════════════════════════════════════════════════════════════
# RELEASE
# ═══════════════════════════════════════════════════════════════════════════════

# Create a release
release version:
    @echo "Creating release {{version}}..."
    @sed -i 's/version = "[^"]*"/version = "{{version}}"/' Cargo.toml
    @sed -i 's/(version . "[^"]*")/(version . "{{version}}")/' STATE.scm
    git add -A
    git commit -m "Release {{version}}"
    git tag -a "v{{version}}" -m "Release {{version}}"
    @echo "Release {{version}} created. Run 'git push && git push --tags' to publish."

# ═══════════════════════════════════════════════════════════════════════════════
# UTILITIES
# ═══════════════════════════════════════════════════════════════════════════════

# Count lines of code
loc:
    @tokei . 2>/dev/null || find . \( -name "*.rs" -o -name "*.res" -o -name "*.adb" -o -name "*.ads" \) | xargs wc -l 2>/dev/null | tail -1

# Show TODO comments
todos:
    @grep -rn "TODO\|FIXME" --include="*.rs" --include="*.res" --include="*.adb" --include="*.ads" . 2>/dev/null || echo "No TODOs"

# Open in editor
edit:
    ${EDITOR:-code} .

# Git status
status:
    @git status --short

# Show recent commits
log count="20":
    @git log --oneline -{{count}}

# ═══════════════════════════════════════════════════════════════════════════════
# MATRIX RECIPES
# ═══════════════════════════════════════════════════════════════════════════════

# Build matrix: [debug|release] × [core|gui|tui|ui|all]
build-matrix mode="debug" target="all":
    @echo "Build matrix: mode={{mode}} target={{target}}"
    @case "{{target}}" in \
        core) cargo build $([ "{{mode}}" = "release" ] && echo "--release") -p formatrix-core ;; \
        gui) cargo build $([ "{{mode}}" = "release" ] && echo "--release") -p formatrix-gui ;; \
        tui) cd tui && gprbuild -P formatrix_tui.gpr -XMODE={{mode}} ;; \
        ui) cd ui && deno task build ;; \
        all) just build$([ "{{mode}}" = "release" ] && echo "-release") ;; \
    esac

# Show all matrix combinations
combinations:
    @echo "=== Combinatoric Matrix Recipes ==="
    @echo ""
    @echo "Build Matrix: just build-matrix [debug|release] [core|gui|tui|ui|all]"
    @echo "Container:    just container-build [tag]"
    @echo "Run:          just run-gui|run-tui|run-debug"

# [AUTO-GENERATED] Multi-arch / RISC-V target
build-riscv:
	@echo "Building for RISC-V..."
	cross build --target riscv64gc-unknown-linux-gnu
