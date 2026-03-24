// SPDX-License-Identifier: PMPL-1.0-or-later
//! Formatrix DB - ArangoDB client for gist library and graph storage
//!
//! Provides:
//! - Document storage for gists
//! - Graph edges for document links and relationships
//! - Tag and collection management
//! - Full-text search across documents

#![forbid(unsafe_code)]
use arangors::client::reqwest::ReqwestClient;
use arangors::{AqlQuery, ClientError, Connection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};

/// Type alias for the database handle with our HTTP client
type Db = Database<ReqwestClient>;

/// Database error types
#[derive(Debug, Error)]
pub enum DbError {
    /// Connection or network error
    #[error("Connection error: {0}")]
    Connection(String),

    /// AQL query error
    #[error("Query error: {0}")]
    Query(String),

    /// Document not found
    #[error("Document not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Collection does not exist
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    /// Constraint violation (unique key, etc.)
    #[error("Constraint violation: {0}")]
    Constraint(String),
}

impl From<ClientError> for DbError {
    fn from(err: ClientError) -> Self {
        match &err {
            ClientError::Arango(arango_err) => {
                let msg = format!("{}", arango_err);
                if msg.contains("not found") || msg.contains("1202") {
                    DbError::NotFound(msg)
                } else if msg.contains("unique constraint") || msg.contains("1210") {
                    DbError::Constraint(msg)
                } else {
                    DbError::Query(msg)
                }
            }
            ClientError::Serde(e) => DbError::Serialization(serde_json::Error::io(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
            )),
            _ => DbError::Connection(err.to_string()),
        }
    }
}

/// Result type for database operations
pub type Result<T> = std::result::Result<T, DbError>;

/// A stored document (gist) in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDocument {
    /// Unique document key (ArangoDB _key)
    #[serde(rename = "_key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Document revision (_rev) for optimistic locking
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,

    /// Document title
    pub title: String,

    /// Raw document content
    pub content: String,

    /// Source format (md, adoc, djot, org, rst, typ, txt)
    pub format: String,

    /// User-defined tags for categorization
    pub tags: Vec<String>,

    /// ISO 8601 creation timestamp
    pub created_at: String,

    /// ISO 8601 last update timestamp
    pub updated_at: String,

    /// Optional parent document key (for hierarchical organization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_key: Option<String>,

    /// Document visibility (private, shared, public)
    #[serde(default)]
    pub visibility: Visibility,
}

/// Document visibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Only owner can access
    #[default]
    Private,
    /// Shared with specific users
    Shared,
    /// Publicly accessible
    Public,
}

/// An edge representing a link between documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLink {
    /// Edge key (optional, auto-generated if not provided)
    #[serde(rename = "_key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Source document reference (documents/{key})
    #[serde(rename = "_from")]
    pub from: String,

    /// Target document reference (documents/{key})
    #[serde(rename = "_to")]
    pub to: String,

    /// Type of relationship
    pub link_type: LinkType,

    /// Optional human-readable label for the link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// ISO 8601 timestamp when link was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Types of relationships between documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    /// Explicit reference/citation
    Reference,
    /// Auto-detected backlink
    Backlink,
    /// Manually marked as related
    Related,
    /// Hierarchical parent relationship
    Parent,
    /// Hierarchical child relationship
    Child,
    /// Document is embedded/included
    Embed,
    /// Deprecated/superseded by target
    Supersedes,
}

/// Tag with usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag name (unique identifier)
    #[serde(rename = "_key")]
    pub name: String,

    /// Number of documents using this tag
    pub count: u64,

    /// ISO 8601 timestamp of last use
    pub last_used: String,
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching document
    pub document: StoredDocument,

    /// Relevance score (higher = more relevant)
    pub score: f64,

    /// Matching snippets with context
    pub snippets: Vec<String>,
}

/// Graph traversal result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Document at this node
    pub document: StoredDocument,

    /// Depth from origin (0 = origin document)
    pub depth: u32,

    /// Inbound links to this document
    pub inbound: Vec<DocumentLink>,

    /// Outbound links from this document
    pub outbound: Vec<DocumentLink>,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// ArangoDB server URL (e.g., "http://localhost:8529")
    pub url: String,

    /// Database name
    pub database: String,

    /// Username for authentication
    pub username: String,

    /// Password for authentication
    pub password: String,

    /// Create collections if they don't exist
    pub auto_create: bool,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8529".to_string(),
            database: "formatrix".to_string(),
            username: "root".to_string(),
            password: String::new(),
            auto_create: true,
        }
    }
}

/// Collection names used by Formatrix
mod collections {
    /// Document collection for gists
    pub const DOCUMENTS: &str = "documents";
    /// Edge collection for links
    pub const LINKS: &str = "links";
    /// Collection for tag statistics
    pub const TAGS: &str = "tags";
    /// Graph name for document relationships
    pub const GRAPH: &str = "doc_graph";
}

/// ArangoDB client for Formatrix document storage
///
/// This is a wrapper around the arangors library that provides
/// a type-safe interface for document and graph operations.
pub struct FormatrixDb {
    /// Connection to ArangoDB (stored as Arc for Clone + Send + Sync)
    conn: Arc<Connection>,

    /// Database name
    db_name: String,

    /// Configuration used for this connection
    config: DbConfig,
}

impl FormatrixDb {
    /// Create a new database client with the given configuration
    #[instrument(skip(config), fields(url = %config.url, database = %config.database))]
    pub async fn connect(config: DbConfig) -> Result<Self> {
        info!("Connecting to ArangoDB");

        let conn = Connection::establish_jwt(&config.url, &config.username, &config.password)
            .await
            .map_err(|e| DbError::Connection(format!("Failed to connect: {}", e)))?;

        let client = Self {
            conn: Arc::new(conn),
            db_name: config.database.clone(),
            config,
        };

        if client.config.auto_create {
            client.ensure_collections().await?;
        }

        info!("Connected to ArangoDB successfully");
        Ok(client)
    }

    /// Get a database handle for running queries
    async fn get_db(&self) -> Result<Db> {
        self.conn
            .db(&self.db_name)
            .await
            .map_err(|e| DbError::Connection(format!("Failed to access database: {}", e)))
    }

    /// Ensure all required collections exist
    #[instrument(skip(self))]
    async fn ensure_collections(&self) -> Result<()> {
        debug!("Ensuring collections exist");
        let db = self.get_db().await?;

        // Check and create document collection
        if db.collection(collections::DOCUMENTS).await.is_err() {
            info!("Creating documents collection");
            db.create_collection(collections::DOCUMENTS)
                .await
                .map_err(|e| {
                    DbError::Query(format!("Failed to create documents collection: {}", e))
                })?;
        }

        // Check and create edge collection via AQL (arangors limitation)
        if db.collection(collections::LINKS).await.is_err() {
            info!("Creating links edge collection");
            warn!("Edge collection creation may require manual setup via ArangoDB UI");
        }

        // Check and create tags collection
        if db.collection(collections::TAGS).await.is_err() {
            info!("Creating tags collection");
            db.create_collection(collections::TAGS)
                .await
                .map_err(|e| DbError::Query(format!("Failed to create tags collection: {}", e)))?;
        }

        Ok(())
    }

    /// Store a new document or update an existing one
    #[instrument(skip(self, doc), fields(title = %doc.title))]
    pub async fn save_document(&self, doc: &StoredDocument) -> Result<String> {
        let db = self.get_db().await?;
        let collection = db
            .collection(collections::DOCUMENTS)
            .await
            .map_err(|_| DbError::CollectionNotFound(collections::DOCUMENTS.to_string()))?;

        let result = if let Some(key) = &doc.key {
            // Update existing document
            debug!("Updating document");
            collection
                .update_document(key, doc.clone(), Default::default())
                .await
                .map_err(|e| DbError::Query(format!("Failed to update document: {}", e)))?
        } else {
            // Insert new document
            debug!("Inserting new document");
            collection
                .create_document(doc.clone(), Default::default())
                .await
                .map_err(|e| DbError::Query(format!("Failed to insert document: {}", e)))?
        };

        let header = result
            .header()
            .ok_or_else(|| DbError::Query("No header in response".to_string()))?;
        let key = header._key.clone();
        info!(key = %key, "Document saved successfully");

        // Update tag statistics
        for tag in &doc.tags {
            self.update_tag_count(tag).await?;
        }

        Ok(key)
    }

    /// Get a document by its key
    #[instrument(skip(self))]
    pub async fn get_document(&self, key: &str) -> Result<StoredDocument> {
        let db = self.get_db().await?;
        let collection = db
            .collection(collections::DOCUMENTS)
            .await
            .map_err(|_| DbError::CollectionNotFound(collections::DOCUMENTS.to_string()))?;

        let doc: StoredDocument = collection
            .document(key)
            .await
            .map_err(|e| DbError::NotFound(format!("Document '{}' not found: {}", key, e)))?
            .document;

        debug!(key = %key, "Document retrieved");
        Ok(doc)
    }

    /// Delete a document by its key
    #[instrument(skip(self))]
    pub async fn delete_document(&self, key: &str) -> Result<()> {
        let db = self.get_db().await?;
        let collection = db
            .collection(collections::DOCUMENTS)
            .await
            .map_err(|_| DbError::CollectionNotFound(collections::DOCUMENTS.to_string()))?;

        collection
            .remove_document::<StoredDocument>(key, Default::default(), Default::default())
            .await
            .map_err(|e| DbError::Query(format!("Failed to delete document: {}", e)))?;

        // Also delete associated links
        let delete_links_aql = AqlQuery::builder()
            .query(r#"
                FOR link IN links
                    FILTER link._from == CONCAT("documents/", @key) OR link._to == CONCAT("documents/", @key)
                    REMOVE link IN links
            "#)
            .bind_var("key", serde_json::json!(key))
            .build();

        let _: Vec<serde_json::Value> = db.aql_query(delete_links_aql).await.unwrap_or_default();

        info!(key = %key, "Document and associated links deleted");
        Ok(())
    }

    /// Search documents by tags (documents must have ALL specified tags)
    #[instrument(skip(self))]
    pub async fn search_by_tags(&self, tags: &[&str]) -> Result<Vec<StoredDocument>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let db = self.get_db().await?;
        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR doc IN documents
                    FILTER LENGTH(INTERSECTION(doc.tags, @tags)) == LENGTH(@tags)
                    SORT doc.updated_at DESC
                    RETURN doc
            "#,
            )
            .bind_var("tags", serde_json::json!(tags))
            .build();

        let results: Vec<StoredDocument> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Tag search failed: {}", e)))?;

        debug!(count = results.len(), "Documents found by tags");
        Ok(results)
    }

    /// Search documents by tag (convenience method for single tag)
    #[instrument(skip(self))]
    pub async fn search_by_tag(&self, tag: &str) -> Result<Vec<StoredDocument>> {
        self.search_by_tags(&[tag]).await
    }

    /// Full-text search across document titles and content
    #[instrument(skip(self))]
    pub async fn search_fulltext(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>> {
        let db = self.get_db().await?;
        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR doc IN documents
                    LET title_match = CONTAINS(LOWER(doc.title), LOWER(@query))
                    LET content_match = CONTAINS(LOWER(doc.content), LOWER(@query))
                    FILTER title_match OR content_match
                    LET score = (title_match ? 2.0 : 0.0) + (content_match ? 1.0 : 0.0)
                    SORT score DESC, doc.updated_at DESC
                    LIMIT @limit
                    RETURN {
                        document: doc,
                        score: score,
                        snippets: content_match ? [SUBSTRING(doc.content, 0, 200)] : []
                    }
            "#,
            )
            .bind_var("query", serde_json::json!(query))
            .bind_var("limit", serde_json::json!(limit))
            .build();

        let results: Vec<SearchResult> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Full-text search failed: {}", e)))?;

        debug!(query = %query, count = results.len(), "Full-text search completed");
        Ok(results)
    }

    /// Get all links for a document (both inbound and outbound)
    #[instrument(skip(self))]
    pub async fn get_links(&self, doc_key: &str) -> Result<Vec<DocumentLink>> {
        let db = self.get_db().await?;
        let doc_id = format!("documents/{}", doc_key);

        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR link IN links
                    FILTER link._from == @doc_id OR link._to == @doc_id
                    RETURN link
            "#,
            )
            .bind_var("doc_id", serde_json::json!(doc_id))
            .build();

        let links: Vec<DocumentLink> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to get links: {}", e)))?;

        debug!(key = %doc_key, count = links.len(), "Links retrieved");
        Ok(links)
    }

    /// Get only outbound links from a document
    #[instrument(skip(self))]
    pub async fn get_outbound_links(&self, doc_key: &str) -> Result<Vec<DocumentLink>> {
        let db = self.get_db().await?;
        let doc_id = format!("documents/{}", doc_key);

        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR link IN links
                    FILTER link._from == @doc_id
                    RETURN link
            "#,
            )
            .bind_var("doc_id", serde_json::json!(doc_id))
            .build();

        let links: Vec<DocumentLink> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to get outbound links: {}", e)))?;

        Ok(links)
    }

    /// Get only inbound links (backlinks) to a document
    #[instrument(skip(self))]
    pub async fn get_backlinks(&self, doc_key: &str) -> Result<Vec<DocumentLink>> {
        let db = self.get_db().await?;
        let doc_id = format!("documents/{}", doc_key);

        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR link IN links
                    FILTER link._to == @doc_id
                    RETURN link
            "#,
            )
            .bind_var("doc_id", serde_json::json!(doc_id))
            .build();

        let links: Vec<DocumentLink> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to get backlinks: {}", e)))?;

        Ok(links)
    }

    /// Add a link between two documents
    #[instrument(skip(self, link))]
    pub async fn add_link(&self, link: &DocumentLink) -> Result<String> {
        let db = self.get_db().await?;
        let collection = db
            .collection(collections::LINKS)
            .await
            .map_err(|_| DbError::CollectionNotFound(collections::LINKS.to_string()))?;

        let result = collection
            .create_document(link.clone(), Default::default())
            .await
            .map_err(|e| DbError::Query(format!("Failed to create link: {}", e)))?;

        let header = result
            .header()
            .ok_or_else(|| DbError::Query("No header in response".to_string()))?;
        let key = header._key.clone();
        info!(key = %key, from = %link.from, to = %link.to, "Link created");
        Ok(key)
    }

    /// Remove a link by its key
    #[instrument(skip(self))]
    pub async fn remove_link(&self, key: &str) -> Result<()> {
        let db = self.get_db().await?;
        let collection = db
            .collection(collections::LINKS)
            .await
            .map_err(|_| DbError::CollectionNotFound(collections::LINKS.to_string()))?;

        collection
            .remove_document::<DocumentLink>(key, Default::default(), Default::default())
            .await
            .map_err(|e| DbError::Query(format!("Failed to remove link: {}", e)))?;

        info!(key = %key, "Link removed");
        Ok(())
    }

    /// Traverse the document graph from a starting point
    #[instrument(skip(self))]
    pub async fn traverse_graph(&self, start_key: &str, depth: u32) -> Result<Vec<GraphNode>> {
        let db = self.get_db().await?;
        let start_id = format!("documents/{}", start_key);

        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR v, e, p IN 0..@depth ANY @start GRAPH @graph
                    OPTIONS { uniqueVertices: "global", uniqueEdges: "path" }
                    LET inbound = (
                        FOR link IN links
                            FILTER link._to == v._id
                            RETURN link
                    )
                    LET outbound = (
                        FOR link IN links
                            FILTER link._from == v._id
                            RETURN link
                    )
                    RETURN {
                        document: v,
                        depth: LENGTH(p.vertices) - 1,
                        inbound: inbound,
                        outbound: outbound
                    }
            "#,
            )
            .bind_var("start", serde_json::json!(start_id))
            .bind_var("depth", serde_json::json!(depth))
            .bind_var("graph", serde_json::json!(collections::GRAPH))
            .build();

        let nodes: Vec<GraphNode> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Graph traversal failed: {}", e)))?;

        debug!(start = %start_key, depth = depth, nodes = nodes.len(), "Graph traversal completed");
        Ok(nodes)
    }

    /// Get all tags with their usage counts
    #[instrument(skip(self))]
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let db = self.get_db().await?;
        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR tag IN tags
                    SORT tag.count DESC
                    RETURN tag
            "#,
            )
            .build();

        let tags: Vec<Tag> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to get tags: {}", e)))?;

        Ok(tags)
    }

    /// Update tag usage count (internal helper)
    async fn update_tag_count(&self, tag_name: &str) -> Result<()> {
        let db = self.get_db().await?;
        let now = chrono::Utc::now().to_rfc3339();

        let aql = AqlQuery::builder()
            .query(
                r#"
                UPSERT { _key: @tag }
                INSERT { _key: @tag, count: 1, last_used: @now }
                UPDATE { count: OLD.count + 1, last_used: @now }
                IN tags
            "#,
            )
            .bind_var("tag", serde_json::json!(tag_name))
            .bind_var("now", serde_json::json!(now))
            .build();

        let _: Vec<serde_json::Value> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to update tag count: {}", e)))?;

        Ok(())
    }

    /// Get recent documents (sorted by updated_at)
    #[instrument(skip(self))]
    pub async fn get_recent(&self, limit: u32) -> Result<Vec<StoredDocument>> {
        let db = self.get_db().await?;
        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR doc IN documents
                    SORT doc.updated_at DESC
                    LIMIT @limit
                    RETURN doc
            "#,
            )
            .bind_var("limit", serde_json::json!(limit))
            .build();

        let docs: Vec<StoredDocument> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to get recent documents: {}", e)))?;

        Ok(docs)
    }

    /// Get documents by format
    #[instrument(skip(self))]
    pub async fn get_by_format(&self, format: &str) -> Result<Vec<StoredDocument>> {
        let db = self.get_db().await?;
        let aql = AqlQuery::builder()
            .query(
                r#"
                FOR doc IN documents
                    FILTER doc.format == @format
                    SORT doc.updated_at DESC
                    RETURN doc
            "#,
            )
            .bind_var("format", serde_json::json!(format))
            .build();

        let docs: Vec<StoredDocument> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to get documents by format: {}", e)))?;

        Ok(docs)
    }

    /// Count total documents
    #[instrument(skip(self))]
    pub async fn count_documents(&self) -> Result<u64> {
        let db = self.get_db().await?;
        let aql = AqlQuery::builder()
            .query("RETURN LENGTH(documents)")
            .build();

        let counts: Vec<u64> = db
            .aql_query(aql)
            .await
            .map_err(|e| DbError::Query(format!("Failed to count documents: {}", e)))?;

        Ok(counts.first().copied().unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_document_serialization() {
        let doc = StoredDocument {
            key: Some("test123".to_string()),
            rev: None,
            title: "Test Document".to_string(),
            content: "# Hello\n\nThis is content.".to_string(),
            format: "md".to_string(),
            tags: vec!["test".to_string(), "example".to_string()],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T12:00:00Z".to_string(),
            parent_key: None,
            visibility: Visibility::Private,
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("\"_key\":\"test123\""));
        assert!(json.contains("\"title\":\"Test Document\""));

        let deserialized: StoredDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, doc.title);
        assert_eq!(deserialized.tags, doc.tags);
    }

    #[test]
    fn test_document_link_serialization() {
        let link = DocumentLink {
            key: None,
            from: "documents/doc1".to_string(),
            to: "documents/doc2".to_string(),
            link_type: LinkType::Reference,
            label: Some("See also".to_string()),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("\"_from\":\"documents/doc1\""));
        assert!(json.contains("\"_to\":\"documents/doc2\""));
        assert!(json.contains("\"link_type\":\"reference\""));
    }

    #[test]
    fn test_link_type_variants() {
        assert_eq!(
            serde_json::to_string(&LinkType::Reference).unwrap(),
            "\"reference\""
        );
        assert_eq!(
            serde_json::to_string(&LinkType::Backlink).unwrap(),
            "\"backlink\""
        );
        assert_eq!(
            serde_json::to_string(&LinkType::Supersedes).unwrap(),
            "\"supersedes\""
        );
    }

    #[test]
    fn test_visibility_default() {
        let vis: Visibility = Default::default();
        assert_eq!(vis, Visibility::Private);
    }

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.url, "http://localhost:8529");
        assert_eq!(config.database, "formatrix");
        assert!(config.auto_create);
    }
}
