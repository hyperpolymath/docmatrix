# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
set shell := ["bash", "-cu"]
set windows-shell := ["bash", "-cu"]
mod provision 'build/just/provision.just'

# Show tasks and provisioning commands
default:
    @just --list --list-submodules

# Install the declared toolchain and run diagnostics
setup: provision::setup
doctor: provision::doctor
heal: provision::heal
dev-shell: provision::dev-shell
toolchain-refresh: provision::toolchain-refresh
config-show: provision::config-show
ai-setup: provision::ai-setup
opsm: provision::opsm
langs: provision::langs
eval: provision::eval
deps: provision::deps
search pattern:
    @just provision::search {{quote(pattern)}}

# Read an audience-specific AI warm-up
ai-warmup who="user":
    #!/usr/bin/env bash
    set -euo pipefail
    case {{quote(who)}} in
      user|dev|maintainer|theorist) cat "docs/onboarding/llm-warmup-{{who}}.adoc" ;;
      *) echo "audience must be user, dev, maintainer or theorist" >&2; exit 2 ;;
    esac

# Build the portable default package set
build:
    cargo build --locked
build-release:
    cargo build --locked --release

# Execute tests, benchmark smoke tests and documentation examples
test:
    cargo test --locked --all-targets
    cargo test --locked --doc

# Measure release builds; retain host/compiler information with the result
bench:
    cargo bench --locked
lint:
    cargo clippy --locked --all-targets -- -D warnings
fmt:
    cargo fmt --all
fmt-check:
    cargo fmt --all --check
doc:
    cargo doc --locked --no-deps

# The same local acceptance sequence used before review
check: fmt-check lint test doc

# Explicit component scopes; optional components fail when prerequisites are absent
build-core:
    cargo build --locked -p formatrix-core
build-data:
    cargo build --locked -p docmatrix-data
build-gui:
    cargo build --locked -p formatrix-gui
build-workspace:
    cargo build --locked --workspace
test-core:
    cargo test --locked -p formatrix-core
test-data:
    cargo test --locked -p docmatrix-data
test-workspace:
    cargo test --locked --workspace --all-targets
test-integration:
    cargo test --locked --workspace -- --ignored
run-gui *args:
    cargo run --locked -p formatrix-gui -- {{args}}
build-tui:
    cargo build --locked --release -p formatrix-core
    cd tui && gprbuild -P formatrix_tui.gpr
run-tui *args: build-tui
    tui/bin/formatrix_tui {{args}}

# Dependency and supply-chain tools must actually run to report success
deps-audit:
    cargo audit
security:
    trivy fs --severity HIGH,CRITICAL --exit-code 1 .
sbom:
    syft dir:. -o spdx-json
guix-build:
    guix build -f guix.scm
guix-shell:
    guix shell -m manifest.scm
docs: doc
quality: check
ci: check
clean:
    cargo clean
status:
    git status --short

# Run the native CLI
run *args:
    cargo run --locked --bin docmatrix -- {{args}}

# Print the package version
version:
    cargo run --locked --bin docmatrix -- --version

# Build the Wolfi image with Podman
container-build:
    podman build --file Containerfile --tag localhost/docmatrix:dev .

# Run without network access, capabilities or a writable root filesystem
container-run *args:
    podman run --rm -i --network=none --read-only --cap-drop=ALL --memory=256m --cpus=1 --pids-limit=64 --security-opt=no-new-privileges localhost/docmatrix:dev {{args}}

# A container smoke test; absence of Podman is a failure
container-test:
    podman run --rm --network=none --read-only --cap-drop=ALL localhost/docmatrix:dev --version

# Inspect first-party Creusot/Pearlite and Verus obligations
proof-status:
    cat verification/PROOF-STATUS.adoc

# Verify the real converter seeds before running an instrumented campaign
fuzz-seeds:
    cargo test --locked --manifest-path fuzz/Cargo.toml --lib
    cargo clippy --locked --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings

# Instrumented campaign; cargo-fuzz and a nightly sanitizer toolchain are required
fuzz target="kyaml":
    #!/usr/bin/env bash
    set -euo pipefail
    case {{quote(target)}} in
      kyaml|json|jcs|dhall) ;;
      *) echo "target must be kyaml, json, jcs or dhall" >&2; exit 2 ;;
    esac
    locked_copy=$(mktemp)
    trap 'rm -f "$locked_copy"' EXIT
    cp fuzz/Cargo.lock "$locked_copy"
    cargo +nightly fetch --locked --manifest-path fuzz/Cargo.toml
    fuzz_host=$(rustc +nightly -vV | sed -n 's/^host: //p')
    test -n "$fuzz_host"
    CARGO_NET_OFFLINE=true cargo +nightly fuzz run --target "$fuzz_host" {{quote(target)}} -- -max_total_time=300 -max_len=16384 -timeout=10 -rss_limit_mb=2048
    cmp "$locked_copy" fuzz/Cargo.lock
