;; STATE.scm - RSR State File
;; SPDX-License-Identifier: MPL-2.0-or-later
;; Copyright (C) 2025 Jonathan D.A. Jewell
;;
;; This file tracks the current state of the project using S-expressions.
;; It is machine-readable and used by RSR tooling for validation.

(state
  (version . "0.1.0")
  (phase . "scaffold")
  (updated . "2026-01-02T00:00:00Z")

  (project
    (name . "formatrix-docs")
    (tier . "2")
    (license . "AGPL-3.0-or-later")
    (languages . ("rust" "ada" "rescript")))

  (compliance
    (rsr . #t)
    (security-hardened . #t)
    (ci-cd . #f)
    (guix-primary . #t)
    (nix-fallback . #t))

  (artifacts
    (binary-gui . "target/release/formatrix-gui")
    (binary-tui . "tui/bin/formatrix-tui")
    (container . "ghcr.io/hyperpolymath/formatrix-docs:latest"))

  (dependencies
    (build
      ("rust" . ">=1.75")
      ("gnat" . ">=12")
      ("gprbuild" . ">=22")
      ("deno" . ">=2.0"))
    (runtime
      ("arangodb" . ">=3.11")
      ("tesseract" . ">=5.0")
      ("hunspell" . ">=1.7")))

  (milestones
    (v0.1.0
      (status . "in-progress")
      (features
        "Format tabs (TXT, MD, ADOC, DJOT, ORG, RST, TYP)"
        "Basic editing"
        "File open/save"
        "Git status integration"
        "Spell check (hunspell)"
        "Print/export to PDF"))
    (v0.2.0
      (status . "planned")
      (features
        "Graph view (document links)"
        "OCR (Tesseract)"
        "TTS/STT (espeak-ng/Vosk)"
        "Pandoc bridge"
        "Agrep fuzzy search"
        "Gist library (ArangoDB)"
        "Permissions manager"
        "Share/send"
        "Nickel pipelines"))
    (v0.3.0
      (status . "planned")
      (features
        "Flying Logic-style reasoning diagrams"
        "Advanced graph analysis"
        "Collaborative editing"))))
