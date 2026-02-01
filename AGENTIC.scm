;; SPDX-License-Identifier: MPL-2.0-or-later
;; AGENTIC.scm - AI agent interaction patterns
;; Copyright (C) 2025 Jonathan D.A. Jewell

(define agentic-config
  `((version . "1.0.0")

    (claude-code
      ((model . "claude-opus-4-5-20251101")
       (tools . ("read" "edit" "bash" "grep" "glob" "task"))
       (permissions . "read-all")))

    (patterns
      ((code-review . "thorough")
       (refactoring . "conservative")
       (testing . "comprehensive")
       (documentation . "asciidoc-preferred")))

    (constraints
      ((languages . ("rescript" "rust" "ada" "nickel" "scheme"))
       (banned . ("typescript" "go" "python" "node" "makefile"))
       (runtime . "deno")
       (package-manager . "deno")))

    (project-specific
      ((format-handlers . "src/formats/*.rs - implement Parser + Renderer traits")
       (tui-widgets . "tui/src/ui/*.ads - Ada widget specs")
       (ui-components . "ui/src/components/*.res - ReScript React components")
       (pipelines . "pipelines/*.ncl - Nickel pipeline definitions")))

    (checkpoint-files
      ((state . "STATE.scm")
       (ecosystem . "ECOSYSTEM.scm")
       (meta . "META.scm")
       (playbook . "PLAYBOOK.scm")
       (agentic . "AGENTIC.scm")
       (neurosym . "NEUROSYM.scm")))))
