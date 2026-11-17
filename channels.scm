;; SPDX-License-Identifier: MPL-2.0
;; channels.scm — the Guix revision this repository's guix.scm and manifest.scm
;; were verified against. Reproduce that exact Guix with:
;;
;;   guix time-machine -C channels.scm -- shell -m manifest.scm
;;
;; Refresh it (after re-verifying) with: just toolchain-refresh
;; Canon pin, verified 2026-09-30 (docker.io/metacall/guix, guix describe).
(list (channel
       (name 'guix)
       (url "https://codeberg.org/guix/guix.git")
       (branch "master")
       (commit "ae77aeb9543de2661d739104bc4d3803d8c6f38b")
       (introduction
        (make-channel-introduction
         "9edb3f66fd807b096b48283debdcddccfea34bad"
         (openpgp-fingerprint
          "BBB0 2DDF 2CEA F6A8 0D1D  E643 A2A0 6DF2 A33A 54FA")))))
