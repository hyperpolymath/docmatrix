// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tauri commands for document operations

use formatrix_core::{Document, ParseConfig, RenderConfig, SourceFormat};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

// =============================================================================
// FD-M12: Document event emission
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentEvent {
    Created {
        hash: String,
        path: String,
        format: String,
        timestamp: f64,
    },
    Modified {
        hash: String,
        old_hash: String,
        path: String,
        format: String,
        timestamp: f64,
    },
    Deleted {
        hash: String,
        path: String,
        timestamp: f64,
    },
    Converted {
        source_hash: String,
        target_hash: String,
        from_format: String,
        to_format: String,
        timestamp: f64,
    },
}

impl DocumentEvent {
    pub fn created(content: &str, path: &str, format: &str) -> Self {
        DocumentEvent::Created {
            hash: hash_content(content),
            path: path.to_string(),
            format: format.to_string(),
            timestamp: current_timestamp(),
        }
    }

    pub fn modified(content: &str, old_content: &str, path: &str, format: &str) -> Self {
        DocumentEvent::Modified {
            hash: hash_content(content),
            old_hash: hash_content(old_content),
            path: path.to_string(),
            format: format.to_string(),
            timestamp: current_timestamp(),
        }
    }

    pub fn deleted(content: &str, path: &str) -> Self {
        DocumentEvent::Deleted {
            hash: hash_content(content),
            path: path.to_string(),
            timestamp: current_timestamp(),
        }
    }

    pub fn converted(source: &str, target: &str, from: &str, to: &str) -> Self {
        DocumentEvent::Converted {
            source_hash: hash_content(source),
            target_hash: hash_content(target),
            from_format: from.to_string(),
            to_format: to.to_string(),
            timestamp: current_timestamp(),
        }
    }
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn current_timestamp() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

// Event log for tracking document changes
static EVENT_LOG: std::sync::LazyLock<std::sync::Mutex<Vec<DocumentEvent>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

/// Emit a document event
pub fn emit_event(event: DocumentEvent) {
    if let Ok(mut log) = EVENT_LOG.lock() {
        log.push(event);
    }
}

/// Get recent document events
#[tauri::command]
pub fn get_document_events(limit: usize) -> Vec<DocumentEvent> {
    if let Ok(log) = EVENT_LOG.lock() {
        log.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}

/// Clear document event log
#[tauri::command]
pub fn clear_document_events() {
    if let Ok(mut log) = EVENT_LOG.lock() {
        log.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub path: Option<String>,
    pub format: String,
    pub modified: bool,
    pub word_count: usize,
    pub char_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentData {
    pub content: String,
    pub meta: DocumentMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub content: String,
    pub warnings: Vec<String>,
}

/// Load a document from the filesystem
#[tauri::command]
pub async fn load_document(path: String) -> Result<DocumentData, String> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Detect format from extension
    let format = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| match ext {
            "txt" => "txt",
            "md" | "markdown" => "md",
            "adoc" | "asciidoc" => "adoc",
            "dj" | "djot" => "djot",
            "org" => "org",
            "rst" => "rst",
            "typ" => "typ",
            _ => "txt",
        })
        .unwrap_or("txt")
        .to_string();

    let word_count = content.split_whitespace().count();
    let char_count = content.chars().count();

    Ok(DocumentData {
        content,
        meta: DocumentMeta {
            path: Some(path),
            format,
            modified: false,
            word_count,
            char_count,
        },
    })
}

/// Save a document to the filesystem
#[tauri::command]
pub async fn save_document(
    path: String,
    content: String,
    format: String,
) -> Result<DocumentMeta, String> {
    tokio::fs::write(&path, &content)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    let word_count = content.split_whitespace().count();
    let char_count = content.chars().count();

    Ok(DocumentMeta {
        path: Some(path),
        format,
        modified: false,
        word_count,
        char_count,
    })
}

/// Convert document content from one format to another
#[tauri::command]
pub async fn convert_to_format(
    content: String,
    from_format: String,
    to_format: String,
) -> Result<ConversionResult, String> {
    use formatrix_core::formats::{MarkdownHandler, PlainTextHandler};
    use formatrix_core::traits::{Parser, Renderer};

    // For now, just return the content as-is if converting to same format
    if from_format == to_format {
        return Ok(ConversionResult {
            content,
            warnings: Vec::new(),
        });
    }

    // Parse source format
    let parse_config = ParseConfig::default();
    let render_config = RenderConfig::default();

    let doc = match from_format.as_str() {
        "txt" => PlainTextHandler::new()
            .parse(&content, &parse_config)
            .map_err(|e| e.to_string())?,
        "md" => MarkdownHandler::new()
            .parse(&content, &parse_config)
            .map_err(|e| e.to_string())?,
        _ => {
            return Err(format!("Unsupported source format: {}", from_format));
        }
    };

    // Render to target format
    let output = match to_format.as_str() {
        "txt" => PlainTextHandler::new()
            .render(&doc, &render_config)
            .map_err(|e| e.to_string())?,
        "md" => MarkdownHandler::new()
            .render(&doc, &render_config)
            .map_err(|e| e.to_string())?,
        _ => {
            return Err(format!("Unsupported target format: {}", to_format));
        }
    };

    Ok(ConversionResult {
        content: output,
        warnings: Vec::new(),
    })
}
