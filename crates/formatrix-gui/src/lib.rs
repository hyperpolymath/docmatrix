// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//! Formatrix GUI - Gossamer desktop application
//!
//! Document editor commands exposed via Gossamer IPC.

#![forbid(unsafe_code)]
pub mod commands;

pub use commands::*;
