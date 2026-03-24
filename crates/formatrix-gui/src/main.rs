// SPDX-License-Identifier: PMPL-1.0-or-later
//! Formatrix Docs - Gossamer desktop application
//!
//! Cross-platform document editor with format tabs.

use gossamer_rs::App;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

fn main() -> Result<(), gossamer_rs::Error> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "formatrix_gui=debug,gossamer=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Formatrix Docs v{}", env!("CARGO_PKG_VERSION"));

    let mut app = App::new("Formatrix Docs", 1200, 800)?;

    // -------------------------------------------------------------------------
    // Register commands — each closure deserializes JSON args, calls business
    // logic in the commands module, and returns a JSON result.
    // -------------------------------------------------------------------------

    app.command("load_document", |payload| {
        let path = payload["path"]
            .as_str()
            .ok_or_else(|| "missing 'path' argument".to_string())?
            .to_string();
        let result = commands::load_document(path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.command("save_document", |payload| {
        let path = payload["path"]
            .as_str()
            .ok_or_else(|| "missing 'path' argument".to_string())?
            .to_string();
        let content = payload["content"]
            .as_str()
            .ok_or_else(|| "missing 'content' argument".to_string())?
            .to_string();
        let format = payload["format"]
            .as_str()
            .ok_or_else(|| "missing 'format' argument".to_string())?
            .to_string();
        let result = commands::save_document(path, content, format)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.command("convert_to_format", |payload| {
        let content = payload["content"]
            .as_str()
            .ok_or_else(|| "missing 'content' argument".to_string())?
            .to_string();
        let from_format = payload["from_format"]
            .as_str()
            .ok_or_else(|| "missing 'from_format' argument".to_string())?
            .to_string();
        let to_format = payload["to_format"]
            .as_str()
            .ok_or_else(|| "missing 'to_format' argument".to_string())?
            .to_string();
        let result = commands::convert_to_format(content, from_format, to_format)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.command("get_document_events", |payload| {
        let limit = payload["limit"].as_u64().unwrap_or(100) as usize;
        let result = commands::get_document_events(limit);
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.command("clear_document_events", |_payload| {
        commands::clear_document_events();
        Ok(serde_json::json!(null))
    });

    app.command("parse_document", |payload| {
        let content = payload["content"]
            .as_str()
            .ok_or_else(|| "missing 'content' argument".to_string())?
            .to_string();
        let format = payload["format"]
            .as_str()
            .ok_or_else(|| "missing 'format' argument".to_string())?
            .to_string();
        let result = commands::parse_document(content, format)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.command("render_document", |payload| {
        let content = payload["content"]
            .as_str()
            .ok_or_else(|| "missing 'content' argument".to_string())?
            .to_string();
        let to_format = payload["to_format"]
            .as_str()
            .ok_or_else(|| "missing 'to_format' argument".to_string())?
            .to_string();
        let result = commands::render_document(content, to_format)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.command("detect_format", |payload| {
        let content = payload["content"]
            .as_str()
            .ok_or_else(|| "missing 'content' argument".to_string())?
            .to_string();
        let result = commands::detect_format(content);
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.command("get_supported_formats", |_payload| {
        let result = commands::get_supported_formats();
        serde_json::to_value(result).map_err(|e| e.to_string())
    });

    app.run();
    Ok(())
}
