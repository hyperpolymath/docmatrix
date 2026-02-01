;; SPDX-License-Identifier: MPL-2.0-or-later
;; PLAYBOOK.scm - Operational runbook
;; Copyright (C) 2025 Jonathan D.A. Jewell

(define playbook
  `((version . "1.0.0")

    (procedures
      ((build
         (("core" . "just build-core")
          ("gui" . "just build-gui")
          ("tui" . "just build-tui")
          ("all" . "just build")))

       (deploy
         (("container" . "just container-build && just container-push")
          ("release" . "just release")))

       (test
         (("unit" . "just test")
          ("integration" . "just test-integration")
          ("e2e" . "just test-e2e")))

       (rollback
         (("container" . "nerdctl pull ghcr.io/hyperpolymath/formatrix-docs:previous")
          ("git" . "git revert HEAD")))

       (debug
         (("gui" . "RUST_LOG=debug cargo run -p formatrix-gui")
          ("tui" . "./tui/bin/formatrix-tui --debug")
          ("core" . "cargo test -p formatrix-core -- --nocapture")))))

    (alerts
      ((build-failure . "Check CI logs, likely dependency issue")
       (container-fail . "Check nerdctl daemon, may need restart")
       (db-connection . "Verify ArangoDB is running on port 8529")))

    (contacts
      ((maintainer . "hyperpolymath@proton.me")
       (security . "See SECURITY.md")))))
