;; SPDX-License-Identifier: MPL-2.0-or-later
;; Formatrix Docs - Guix Package Definition
;; Copyright (C) 2025 Jonathan D.A. Jewell

(define-module (formatrix)
  #:use-module (guix packages)
  #:use-module (guix gexp)
  #:use-module (guix git-download)
  #:use-module (guix build-system cargo)
  #:use-module (guix build-system gnat)
  #:use-module ((guix licenses) #:prefix license:)
  #:use-module (gnu packages rust)
  #:use-module (gnu packages rust-apps)
  #:use-module (gnu packages ada)
  #:use-module (gnu packages ncurses))

(define-public formatrix-core
  (package
    (name "formatrix-core")
    (version "0.1.0")
    (source
      (origin
        (method git-fetch)
        (uri (git-reference
               (url "https://github.com/hyperpolymath/formatrix-docs")
               (commit (string-append "v" version))))
        (file-name (git-file-name name version))
        (sha256 (base32 "0000000000000000000000000000000000000000000000000000"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-comrak" ,rust-comrak)
         ("rust-jotdown" ,rust-jotdown)
         ("rust-orgize" ,rust-orgize)
         ("rust-serde" ,rust-serde)
         ("rust-thiserror" ,rust-thiserror))))
    (home-page "https://github.com/hyperpolymath/formatrix-docs")
    (synopsis "Core AST and format conversion for Formatrix Docs")
    (description
      "Unified AST and bidirectional conversion between document formats:
Markdown, AsciiDoc, Djot, Org-mode, reStructuredText, Typst, and plain text.")
    (license license:agpl3+)))

(define-public formatrix-tui
  (package
    (name "formatrix-tui")
    (version "0.1.0")
    (source
      (origin
        (method git-fetch)
        (uri (git-reference
               (url "https://github.com/hyperpolymath/formatrix-docs")
               (commit (string-append "v" version))))
        (file-name (git-file-name name version))
        (sha256 (base32 "0000000000000000000000000000000000000000000000000000"))))
    (build-system gnat-build-system)
    (arguments
      `(#:gpr-file "tui/formatrix_tui.gpr"))
    (inputs
      (list ncurses gnat))
    (propagated-inputs
      (list formatrix-core))
    (home-page "https://github.com/hyperpolymath/formatrix-docs")
    (synopsis "Terminal UI for Formatrix Docs")
    (description
      "Ada-based terminal user interface for Formatrix Docs with format tabs,
editor buffer, and ncurses-based rendering.  Safety-critical text handling.")
    (license license:agpl3+)))

(define-public formatrix-gui
  (package
    (name "formatrix-gui")
    (version "0.1.0")
    (source
      (origin
        (method git-fetch)
        (uri (git-reference
               (url "https://github.com/hyperpolymath/formatrix-docs")
               (commit (string-append "v" version))))
        (file-name (git-file-name name version))
        (sha256 (base32 "0000000000000000000000000000000000000000000000000000"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-inputs
        (("rust-tauri" ,rust-tauri)
         ("rust-serde" ,rust-serde)
         ("rust-tokio" ,rust-tokio))))
    (propagated-inputs
      (list formatrix-core))
    (home-page "https://github.com/hyperpolymath/formatrix-docs")
    (synopsis "Desktop and mobile GUI for Formatrix Docs")
    (description
      "Tauri 2.0-based graphical interface for Formatrix Docs.  Supports
desktop (Linux, macOS, Windows) and mobile (Android, iOS) platforms.")
    (license license:agpl3+)))
