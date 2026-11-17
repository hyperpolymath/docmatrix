;; SPDX-License-Identifier: MPL-2.0
;; Guix provides the shell; mise supplies the exact modern Rust toolchain.
(specifications->manifest
 (list "bash" "git" "coreutils" "gcc-toolchain" "pkg-config" "openssl"
       "nss-certs" "mise" "just" "shellcheck"))
