<!--
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# Changelog

All notable changes to `docmatrix` will be documented in this file.

This file is generated from conventional commits by the
[`changelog-reusable.yml`](https://github.com/hyperpolymath/standards/blob/main/.github/workflows/changelog-reusable.yml)
workflow (`hyperpolymath/standards#206`). Adopt the workflow in this repo's CI to keep this file in sync automatically — see
[`templates/cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml)
for the canonical config.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- feat(crg): add crg-grade and crg-badge justfile recipes
- feat: add k9iser.toml and generate K9 contracts
- feat: add stapeln.toml container definition
- feat: deploy UX Manifesto infrastructure
- feat: replace Tauri with Gossamer — gossamer-rs backend
- feat: Gossamer migration — RuntimeBridge, gossamer.conf.json, Tauri→Gossamer conversion
- feat: add CLADE.a2ml — clade taxonomy declaration
- feat: add mirror.yml workflow for GitLab/Bitbucket mirroring
- feat: absorb docmatrix FFI extras from zig-ffi monorepo
- feat: customize fuzz target with repo-specific logic

### Fixed

- fix(ci): bump a2ml/k9-validate-action pins to canonical (standards#85) (#18)
- fix(ci): sync hypatia-scan.yml to canonical (kill cd-scanner build drift) (#17)
- fix(ci): adopt canonical hypatia-scan.yml (env.HOME/scanner-layout + Comment-step gate) (#16)
- fix(ci): Phase-2 fleet submission must not fail the security gate (#15)
- fix(ci): hypatia-scan workdir (${{ env.HOME }} resolves empty) (#14)
- fix(ci): hypatia-scan.yml -- pass GITHUB_TOKEN, use --exit-zero (hyperpolymath/hypatia#213) (#9)
- fix(ci): bump erlef/setup-beam SHA for ubuntu24 runner support (#10)
- fix(ci): repair YAML block-scalar in workflow-linter Check Permissions step (#11)
- fix(ci): move secret-scanner Cargo.toml gate from job-level if: to step-level (#12)
- fix: correct test compilation errors and achieve passing status

### Changed

- refactor: migrate 6SCM → 6A2 (.scm → .a2ml format)

### Documentation

- docs: substantive CRG C annotation (EXPLAINME.adoc)
- docs: add EXPLAINME.adoc — prove-it file backing README claims
- docs: add SECURITY.md for vulnerability reporting
- docs: add checkpoint files for state tracking
- docs: rename to DocMatrix

### CI

- ci(rust): convert rust-ci.yml to thin wrapper (standards#174) (#22)
- ci: redistribute concurrency-cancel guard to read-only check workflows (#20)
- ci(secret-scanner): drop duplicate --fail from trufflehog extra_args (#8)
- ci: bump actions/upload-artifact SHA to current v4 (#7)
- ci: SHA-pin hyperpolymath validate-actions in dogfood-gate

## Pre-history

Prior commits to this file's introduction are recorded in git history but not formally classified into Keep-a-Changelog sections. To backfill, run `git cliff -o CHANGELOG.md` locally using the canonical [`cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml) — this is one-shot mechanical work.

---

<!-- This file was seeded by the 2026-05-26 estate tech-debt audit follow-up (Row-2 Phase 3); see [`hyperpolymath/standards/docs/audits/2026-05-26-estate-documentation-debt.md`](https://github.com/hyperpolymath/standards/blob/main/docs/audits/2026-05-26-estate-documentation-debt.md). -->
