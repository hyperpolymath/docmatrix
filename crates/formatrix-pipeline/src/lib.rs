// SPDX-License-Identifier: PMPL-1.0-or-later
//! Formatrix Pipeline - Nickel-based content transformation engine
//!
//! Pipelines define content transformations for import/export:
//! - Input: Source content or AST
//! - Steps: Ordered list of transforms
//! - Output: Target format and filename pattern

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Nickel evaluation error: {0}")]
    Evaluation(String),

    #[error("Transform not found: {0}")]
    TransformNotFound(String),

    #[error("Invalid pipeline configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PipelineError>;

/// A pipeline definition (matches Nickel schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub input: PipelineInput,
    pub steps: Vec<PipelineStep>,
    pub output: PipelineOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipelineInput {
    /// Raw source text
    Text,
    /// Parsed AST
    Ast,
    /// File path
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PipelineStep {
    /// Add table of contents
    AddToc { depth: u8 },
    /// Resolve internal links
    ResolveLinks,
    /// Render to a format
    Render { format: String },
    /// Convert to output format
    Convert { format: String, engine: Option<String> },
    /// Custom Nickel transform
    Custom { script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    pub format: String,
    pub filename: String,
}

/// Pipeline executor
pub struct PipelineExecutor {
    pipelines: std::collections::HashMap<String, Pipeline>,
}

impl PipelineExecutor {
    pub fn new() -> Self {
        Self {
            pipelines: std::collections::HashMap::new(),
        }
    }

    /// Load a pipeline from a Nickel file
    pub fn load_pipeline(&mut self, _path: &std::path::Path) -> Result<()> {
        // TODO: Parse Nickel file and register pipeline
        Ok(())
    }

    /// Execute a pipeline
    pub fn execute(
        &self,
        pipeline_name: &str,
        input: &str,
    ) -> Result<String> {
        let _pipeline = self.pipelines.get(pipeline_name).ok_or_else(|| {
            PipelineError::TransformNotFound(pipeline_name.to_string())
        })?;

        // TODO: Execute pipeline steps
        Ok(input.to_string())
    }
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}
