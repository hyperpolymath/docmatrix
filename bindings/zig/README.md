<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Zig FFI Bindings

The Zig FFI bindings have been moved to a separate repository for reusability:

**https://github.com/hyperpolymath/zig-formatrix-ffi**

The files in this directory are kept for local development convenience.

## Building formatrix-core with FFI support

```bash
cd ../..
cargo build --release --features ffi
```

## Using the bindings

See the zig-formatrix-ffi repository for full documentation and usage examples.
