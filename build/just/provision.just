# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
#
# provision.just — the estate provisioning contract, as a just MODULE.
#
# Canon: hyperpolymath/standards 3-practice/provisioning/PROVISIONING-STANDARD.adoc
# Loaded from the root Justfile with:
#
#     mod provision 'build/just/provision.just'
#
# A module (not `import`) so these recipes live under `provision::` and never
# collide with a repo's own `doctor`, `heal` or `test`. The root Justfile then
# exposes each contract name ONCE — either its own recipe, or a one-line
# delegation such as `doctor: provision::doctor` that `provision-set mint`
# adds only when the root has no recipe of that name.
#
# requires: just >= 1.42.0 (root recipes depend on module recipes; measured 2026-09-30: 1.41 rejects `x: m::x`, 1.42 accepts)

root := justfile_directory()
lib := "PROVISION_ROOT=" + quote(root) + " bash " + quote(source_directory() / "provision-lib.sh")

# List the provisioning recipes
default:
    @just --list provision

# Install every tool and dependency this repo needs, then run doctor
setup:
    @{{lib}} setup

# Diagnose the environment: PASS/WARN/FAIL with codes; exits 1 on any FAIL
doctor:
    @{{lib}} doctor

# Apply safe, reversible fixes, then re-run doctor
heal:
    @{{lib}} heal

# Enter a shell with every tool on PATH (guix shell -m manifest.scm, else mise)
dev-shell:
    @{{lib}} dev-shell

# Weekly: bump mise 'latest' resolutions, re-lock, re-pin guix channels
toolchain-refresh:
    @{{lib}} toolchain-refresh

# Print (and copy) the one line to give any AI to install this repo
ai-setup:
    @{{lib}} ai-setup

# Print (and copy) the warm-up for an AI session: user, dev or maintainer
ai-warmup who="user":
    @{{lib}} ai-warmup {{quote(who)}}

# Run test + bench, time both, and write .eval/<timestamp>.txt
eval:
    @{{lib}} eval

# Show where this repo's configuration and toolchain are declared
config-show:
    @{{lib}} config-show

# How to install this repo with OPSM (odds-and-sods package manager)
opsm:
    @{{lib}} opsm

# Print the languages detected from marker files
langs:
    @{{lib}} langs

# Approximate search across tracked files (agrep, else git grep)
search pattern:
    @{{lib}} search {{quote(pattern)}}

# Per-language defaults for the contract verbs, reached as `just provision::<verb>`
# or through a root recipe that delegates (`build: provision::build`). A root
# recipe with its own body is the repo-specific override.

# Build with every detected language's toolchain
build:
    @{{lib}} lang-run build

# Run every detected language's tests
test:
    @{{lib}} lang-run test

# Run every detected language's benchmarks
bench:
    @{{lib}} lang-run bench

# Lint every detected language
lint:
    @{{lib}} lang-run lint

# Format every detected language's sources
fmt:
    @{{lib}} lang-run fmt

# Check formatting without rewriting anything (the CI form of fmt)
fmt-check:
    @{{lib}} lang-run fmt-check

# Run the project (N/A for libraries)
run:
    @{{lib}} lang-run run

# Fetch every detected language's dependencies
deps:
    @{{lib}} lang-run deps
