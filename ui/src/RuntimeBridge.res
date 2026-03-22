// SPDX-License-Identifier: PMPL-1.0-or-later

/// RuntimeBridge — Unified IPC bridge for Formatrix Docs (docmatrix).
///
/// Detects the available runtime (Gossamer, Tauri, or browser-only) and
/// dispatches `invoke` calls to the appropriate backend. This allows all
/// command modules to use a single import instead of binding directly
/// to `@tauri-apps/api/core`.
///
/// Priority order:
///   1. Gossamer (`window.__gossamer_invoke`)  — own stack, preferred
///   2. Tauri    (`window.__TAURI_INTERNALS__`) — legacy, transition
///   3. Browser  (direct HTTP fetch)            — development fallback

// ---------------------------------------------------------------------------
// Raw external bindings — exactly one of these will be available at runtime
// ---------------------------------------------------------------------------

/// Gossamer IPC: injected by gossamer_channel_open() into the webview.
%%raw(`
function isGossamerRuntime() {
  return typeof window !== 'undefined'
    && typeof window.__gossamer_invoke === 'function';
}
`)
@val external isGossamerRuntime: unit => bool = "isGossamerRuntime"

%%raw(`
function gossamerInvoke(cmd, args) {
  return window.__gossamer_invoke(cmd, args);
}
`)
@val external gossamerInvoke: (string, 'a) => promise<'b> = "gossamerInvoke"

/// Tauri IPC: injected by the Tauri runtime into the webview.
%%raw(`
function isTauriRuntime() {
  return typeof window !== 'undefined'
    && window.__TAURI_INTERNALS__ != null
    && !window.__TAURI_INTERNALS__.__BROWSER_SHIM__;
}
`)
@val external isTauriRuntime: unit => bool = "isTauriRuntime"

@module("@tauri-apps/api/core")
external tauriInvoke: (string, 'a) => promise<'b> = "invoke"

// ---------------------------------------------------------------------------
// Unified invoke — detects runtime and dispatches
// ---------------------------------------------------------------------------

/// The runtime currently in use. Cached after first detection for performance.
type runtime =
  | Gossamer
  | Tauri
  | BrowserOnly

%%raw(`
var _detectedRuntime = null;
function detectRuntime() {
  if (_detectedRuntime !== null) return _detectedRuntime;
  if (typeof window !== 'undefined' && typeof window.__gossamer_invoke === 'function') {
    _detectedRuntime = 'gossamer';
  } else if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__ != null && !window.__TAURI_INTERNALS__.__BROWSER_SHIM__) {
    _detectedRuntime = 'tauri';
  } else {
    _detectedRuntime = 'browser';
  }
  return _detectedRuntime;
}
`)
@val external detectRuntimeRaw: unit => string = "detectRuntime"

/// Detect and return the current runtime.
let detectRuntime = (): runtime => {
  switch detectRuntimeRaw() {
  | "gossamer" => Gossamer
  | "tauri" => Tauri
  | _ => BrowserOnly
  }
}

/// Invoke a backend command through whatever runtime is available.
let invoke = (cmd: string, args: 'a): promise<'b> => {
  if isGossamerRuntime() {
    gossamerInvoke(cmd, args)
  } else if isTauriRuntime() {
    tauriInvoke(cmd, args)
  } else {
    Promise.reject(
      JsError.throwWithMessage(
        `No desktop runtime — "${cmd}" requires Gossamer or Tauri`,
      ),
    )
  }
}

/// Check whether any desktop runtime is available.
let hasDesktopRuntime = (): bool => {
  isGossamerRuntime() || isTauriRuntime()
}

/// Get a human-readable name for the current runtime.
let runtimeName = (): string => {
  switch detectRuntime() {
  | Gossamer => "Gossamer"
  | Tauri => "Tauri"
  | BrowserOnly => "Browser"
  }
}

// ---------------------------------------------------------------------------
// Dialog abstraction — Gossamer dialogs vs Tauri plugin-dialog
// ---------------------------------------------------------------------------

module Dialog = {
  @module("@tauri-apps/plugin-dialog")
  external tauriOpenRaw: JSON.t => promise<Nullable.t<JSON.t>> = "open"

  @module("@tauri-apps/plugin-dialog")
  external tauriSaveRaw: JSON.t => promise<Nullable.t<JSON.t>> = "save"

  /// Open a file picker dialog.
  let open = (opts: JSON.t): promise<Nullable.t<JSON.t>> => {
    if isGossamerRuntime() {
      gossamerInvoke("__gossamer_dialog_open", opts)
    } else if isTauriRuntime() {
      tauriOpenRaw(opts)
    } else {
      Promise.reject(
        JsError.throwWithMessage(
          "No desktop runtime — file dialogs require Gossamer or Tauri",
        ),
      )
    }
  }

  /// Open a save dialog.
  let save = (opts: JSON.t): promise<Nullable.t<JSON.t>> => {
    if isGossamerRuntime() {
      gossamerInvoke("__gossamer_dialog_save", opts)
    } else if isTauriRuntime() {
      tauriSaveRaw(opts)
    } else {
      Promise.reject(
        JsError.throwWithMessage(
          "No desktop runtime — save dialogs require Gossamer or Tauri",
        ),
      )
    }
  }
}

// ---------------------------------------------------------------------------
// Filesystem abstraction — Gossamer fs vs Tauri plugin-fs
// ---------------------------------------------------------------------------

module Fs = {
  @module("@tauri-apps/plugin-fs")
  external tauriReadTextFileRaw: string => promise<string> = "readTextFile"

  @module("@tauri-apps/plugin-fs")
  external tauriWriteTextFileRaw: (string, string) => promise<unit> = "writeTextFile"

  /// Read a text file from the local filesystem.
  let readTextFile = (path: string): promise<string> => {
    if isGossamerRuntime() {
      gossamerInvoke("__gossamer_fs_read_text", {"path": path})
    } else if isTauriRuntime() {
      tauriReadTextFileRaw(path)
    } else {
      Promise.reject(
        JsError.throwWithMessage(
          "No desktop runtime — filesystem access requires Gossamer or Tauri",
        ),
      )
    }
  }

  /// Write a text file to the local filesystem.
  let writeTextFile = (path: string, contents: string): promise<unit> => {
    if isGossamerRuntime() {
      gossamerInvoke("__gossamer_fs_write_text", {"path": path, "contents": contents})
    } else if isTauriRuntime() {
      tauriWriteTextFileRaw(path, contents)
    } else {
      Promise.reject(
        JsError.throwWithMessage(
          "No desktop runtime — filesystem access requires Gossamer or Tauri",
        ),
      )
    }
  }
}

// ---------------------------------------------------------------------------
// Utility — decode dialog path from either runtime's response format
// ---------------------------------------------------------------------------

/// Decode a dialog result into a file path string.
/// Handles both single-path (String) and multi-path (Array) responses.
let decodeDialogPath = (value: JSON.t): option<string> => {
  switch JSON.Classify.classify(value) {
  | String(path) => Some(path)
  | Array(arr) =>
    switch Array.get(arr, 0) {
    | Some(item) =>
      switch JSON.Classify.classify(item) {
      | String(s) => Some(s)
      | _ => None
      }
    | None => None
    }
  | _ => None
  }
}
