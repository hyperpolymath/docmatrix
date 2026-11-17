;; SPDX-License-Identifier: MPL-2.0
;; SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
;;
;; guix.scm — builds docmatrix with Guix once the acceptance gates in docs/ASSURANCE.adoc pass.
;;
;;   guix build -f guix.scm          # the package
;;   guix shell -D -f guix.scm       # its build environment
;;
;; Crate inputs live in build/guix/crates.scm, generated from Cargo.lock by
;;   guix import crate --lockfile=Cargo.lock docmatrix
;; `just toolchain-refresh` regenerates it; never hand-edit it.
(use-modules (guix packages) (guix gexp)
             (guix build-system cargo)
             (guix git-download)
             (srfi srfi-13)
             ((guix licenses) #:prefix license:))

;; The repository root: this file sits at the root or in build/ (PROVISIONING-STANDARD §1).
(define %source-dir
  (let ((d (dirname (current-filename))))
    (if (string=? (basename d) "build") (dirname d) d)))
(load (string-append %source-dir "/build/guix/crates.scm"))   ; defines %crate-inputs

;; Build products and caches never enter the source (git-predicate is not used:
;; it breaks on tarball checkouts and on bind-mounted trees).
(define %ignored
  '(".git" "target" ".eval" "node_modules" "_build" "deps" "zig-out" ".zig-cache" "dist-newstyle" "secrets"))

(package
  (name "docmatrix")
  (version "0.1.0")
  (source (local-file %source-dir "docmatrix-checkout"
                      #:recursive? #t
                      #:select? (lambda (file stat)
                                  (let ((name (basename file)))
                                    (not (or (member name %ignored)
                                             (string=? name ".env")
                                             (string-prefix? ".env." name)
                                             (string-suffix? ".pem" name)
                                             (string-suffix? ".key" name)))))))
  (build-system cargo-build-system)
  (arguments (list #:install-source? #f
                   #:cargo-install-paths #~(list "crates/docmatrix-data")))
  (inputs %crate-inputs)
  (home-page "https://github.com/hyperpolymath/docmatrix")
  (synopsis "Document conversion infrastructure, with a separate structured-data boundary shared with kyq.")
  (description "Document conversion infrastructure, with a separate structured-data boundary shared with kyq.")
  (license license:mpl2.0))
