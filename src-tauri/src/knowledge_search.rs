//! Enhanced Knowledge Base Search Module
//!
//! Provides realistic knowledge base search functionality for the Tauri application.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Knowledge base document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub knowledge_base_id: String,
    pub embeddings: Option<Vec<f32>>,
}

/// Knowledge base definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub document_count: usize,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub tags: Option<Vec<String>>,
    pub filters: Option<HashMap<String, Value>>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: KnowledgeDocument,
    pub score: f32,
    pub highlights: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query: String,
    pub took_ms: u64,
    pub knowledge_base_id: String,
    pub pagination: Option<PaginationInfo>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Knowledge base search engine
pub struct KnowledgeSearchEngine {
    knowledge_bases: HashMap<String, KnowledgeBase>,
    documents: HashMap<String, KnowledgeDocument>,
    search_index: HashMap<String, Vec<String>>, // Simple inverted index
}

impl KnowledgeSearchEngine {
    pub fn new() -> Self {
        Self {
            knowledge_bases: HashMap::new(),
            documents: HashMap::new(),
            search_index: HashMap::new(),
        }
    }

    /// Initialize the knowledge search engine
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize with sample knowledge bases
        self.initialize_sample_knowledge_bases();
        Ok(())
    }

    /// Initialize sample knowledge bases and documents
    fn initialize_sample_knowledge_bases(&mut self) {
        // Create knowledge bases
        let kb_tech = KnowledgeBase {
            id: "kb-technology".to_string(),
            name: "Technology Knowledge Base".to_string(),
            description: "Documentation about programming languages, frameworks, and technologies".to_string(),
            document_count: 0,
            tags: vec!["technology".to_string(), "programming".to_string(), "documentation".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::from([
                ("category".to_string(), Value::String("technical".to_string())),
                ("language".to_string(), Value::String("english".to_string())),
            ]),
        };

        let kb_business = KnowledgeBase {
            id: "kb-business".to_string(),
            name: "Business Processes".to_string(),
            description: "Standard operating procedures and business guidelines".to_string(),
            document_count: 0,
            tags: vec!["business".to_string(), "procedures".to_string(), "guidelines".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::from([
                ("category".to_string(), Value::String("business".to_string())),
                ("language".to_string(), Value::String("english".to_string())),
            ]),
        };

        self.knowledge_bases.insert(kb_tech.id.clone(), kb_tech);
        self.knowledge_bases.insert(kb_business.id.clone(), kb_business);

        // Add sample documents
        self.add_sample_documents();
    }

    /// Add sample documents to knowledge bases
    fn add_sample_documents(&mut self) {
        let tech_docs = vec![
            KnowledgeDocument {
                id: "doc-rust-basics".to_string(),
                title: "Rust Programming Basics".to_string(),
                content: "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It's designed for performance and reliability, with features like ownership and borrowing that prevent entire classes of bugs. Rust's zero-cost abstractions make it suitable for systems programming, embedded systems, and web services.",
                summary: Some("Introduction to Rust programming language fundamentals".to_string()),
                tags: vec!["rust".to_string(), "programming".to_string(), "systems".to_string(), "memory-safety".to_string()],
                metadata: HashMap::from([
                    ("difficulty".to_string(), Value::String("beginner".to_string())),
                    ("read_time_minutes".to_string(), Value::Number(5.0.into())),
                ]),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                knowledge_base_id: "kb-technology".to_string(),
                embeddings: None,
            },
            KnowledgeDocument {
                id: "doc-async-programming".to_string(),
                title: "Asynchronous Programming in Rust".to_string(),
                content: "Async programming in Rust is made easy with the async/await syntax. The async keyword converts a block of code into a state machine that can be paused and resumed. The await keyword is used within async functions to wait for the completion of asynchronous operations. Rust's ownership model ensures thread safety even in concurrent environments.",
                summary: Some("Guide to using async/await for concurrent programming in Rust".to_string()),
                tags: vec!["rust".to_string(), "async".to_string(), "concurrency".to_string(), "futures".to_string()],
                metadata: HashMap::from([
                    ("difficulty".to_string(), Value::String("intermediate".to_string())),
                    ("read_time_minutes".to_string(), Value::Number(8.0.into())),
                ]),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                knowledge_base_id: "kb-technology".to_string(),
                embeddings: None,
            },
            KnowledgeDocument {
                id: "doc-web-frameworks".to_string(),
                title: "Modern Web Frameworks Comparison".to_string(),
                content: "Modern web development offers numerous frameworks like React, Vue, Angular, Svelte, and Next.js. React focuses on component-based architecture and virtual DOM for performance. Vue provides a progressive framework with excellent documentation. Angular offers a full-featured framework with TypeScript support. Svelte compiles away to vanilla JavaScript for optimal performance. Next.js provides React-based full-stack framework with SSR capabilities.",
                summary: Some("Comparison of popular JavaScript web frameworks".to_string()),
                tags: vec!["javascript".to_string(), "react".to_string(), "vue".to_string(), "angular".to_string(), "svelte".to_string(), "nextjs".to_string(), "frontend".to_string(), "web-development".to_string()],
                metadata: HashMap::from([
                    ("difficulty".to_string(), Value::String("intermediate".to_string())),
                    ("read_time_minutes".to_string(), Value::Number(12.0.into())),
                ]),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                knowledge_base_id: "kb-technology".to_string(),
                embeddings: None,
            },
        ];

        let business_docs = vec![
            KnowledgeDocument {
                id: "doc-onboarding".to_string(),
                title: "Employee Onboarding Process".to_string(),
                content: "The employee onboarding process should begin before the employee's first day and continue through their first week. Key components include: workstation setup, account creation, team introductions, initial training sessions, assignment of a mentor, and setting of initial goals. A structured onboarding process helps new hires become productive more quickly and feel welcomed into the organization.",
                summary: Some("Comprehensive guide to employee onboarding process".to_string()),
                tags: vec!["onboarding".to_string(), "hr".to_string(), "employees".to_string(), "process".to_string(), "management".to_string()],
                metadata: HashMap::from([
                    ("category".to_string(), Value::String("hr".to_string())),
                    ("read_time_minutes".to_string(), Value::Number(10.0.into())),
                ]),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                knowledge_base_id: "kb-business".to_string(),
                embeddings: None,
            },
            KnowledgeDocument {
                id: "doc-meeting-protocol".to_string(),
                title: "Meeting Protocol Guidelines".to_string(),
                content: "Effective meetings require clear protocols. Always send an agenda at least 24 hours in advance. Start meetings on time and end on time. Assign a timekeeper and note-taker. Keep phones on silent during meetings. Follow up with action items within 24 hours. Create and distribute meeting notes promptly. Avoid unnecessary meetings and encourage email or chat for simple communications.",
                summary: Some("Best practices for conducting productive meetings".to_string()),
                tags: vec!["meetings".to_string(), "protocol".to_string(), "productivity".to_string(), "guidelines".to_string(), "communication".to_string()],
                metadata: HashMap::from([
                    ("category".to_string(), Value::String("business".to_string())),
                    ("read_time_minutes".to_string(), Value::Number(6.0.into())),
                ]),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                knowledge_base_id: "kb-business".to_string(),
                embeddings: None,
            },
        ];

        // Add tech documents
        for doc in tech_docs {
            self.add_document(doc);
        }

        // Add business documents
        for doc in business_docs {
            self.add_document(doc);
        }
    }

    /// Add a document to the search engine
    fn add_document(&mut self, document: KnowledgeDocument) {
        // Update document count in knowledge base
        if let Some(kb) = self.knowledge_bases.get_mut(&document.knowledge_base_id) {
            kb.document_count += 1;
            kb.updated_at = Utc::now();
        }

        // Build search index
        let content_words = self.extract_words(&document.content);
        let title_words = self.extract_words(&document.title);
        let summary_words = self.extract_words(&document.summary.as_deref().unwrap_or(""));

        for word in content_words.into_iter().chain(title_words).chain(summary_words) {
            self.search_index
                .entry(word.to_lowercase())
                .or_insert_with(Vec::new)
                .push(document.id.clone());
        }

        self.documents.insert(document.id.clone(), document);
    }

    /// Extract words from text for indexing
    fn extract_words(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .chars()
            .collect::<String>()
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Create a new knowledge base
    pub fn create_knowledge_base(
        &mut self,
        name: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<&KnowledgeBase, String> {
        if name.is_empty() {
            return Err("Knowledge base name cannot be empty".to_string());
        }

        let kb = KnowledgeBase {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            document_count: 0,
            tags,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        self.knowledge_bases.insert(kb.id.clone(), kb.clone());
        Ok(self.knowledge_bases.get(&kb.id).unwrap())
    }

    /// List all knowledge bases
    pub fn list_knowledge_bases(&self) -> Vec<&KnowledgeBase> {
        self.knowledge_bases.values().collect()
    }

    /// Get knowledge base by ID
    pub fn get_knowledge_base(&self, kb_id: &str) -> Option<&KnowledgeBase> {
        self.knowledge_bases.get(kb_id)
    }

    /// Update knowledge base
    pub fn update_knowledge_base(
        &mut self,
        kb_id: &str,
        mut kb: KnowledgeBase,
    ) -> Result<&KnowledgeBase, String> {
        if !self.knowledge_bases.contains_key(kb_id) {
            return Err("Knowledge base not found".to_string());
        }

        kb.id = kb_id.to_string();
        kb.updated_at = Utc::now();

        self.knowledge_bases.insert(kb_id.to_string(), kb);
        Ok(self.knowledge_bases.get(kb_id).unwrap())
    }

    /// Delete knowledge base
    pub fn delete_knowledge_base(&mut self, kb_id: &str) -> Option<KnowledgeBase> {
        // Remove all documents associated with this knowledge base
        let documents_to_remove: Vec<String> = self.documents
            .values()
            .filter(|doc| doc.knowledge_base_id == kb_id)
            .map(|doc| doc.id.clone())
            .collect();

        for doc_id in &documents_to_remove {
            self.documents.remove(doc_id);
        }

        // Remove from search index
        self.search_index.retain(|_, doc_ids| {
            !doc_ids.iter().any(|id| documents_to_remove.contains(id))
        });

        self.knowledge_bases.remove(kb_id)
    }

    /// Search across all knowledge bases
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse, String> {
        let start_time = std::time::Instant::now();

        // Parse query into individual terms
        let query_terms: Vec<String> = self.extract_words(&query.query);

        if query_terms.is_empty() {
            return Err("Search query cannot be empty".to_string());
        }

        // Find matching documents
        let mut matching_docs: HashMap<String, (usize, f32)> = HashMap::new();

        for term in &query_terms {
            if let Some(doc_ids) = self.search_index.get(term) {
                for doc_id in doc_ids {
                    let count = matching_docs.entry(doc_id.clone()).or_insert((0, 0.0));
                    count.0 += 1;
                    count.1 += 1.0; // Simple scoring boost
                }
            }
        }

        // Convert to search results and sort by score
        let mut results: Vec<SearchResult> = matching_docs
            .into_iter()
            .filter_map(|(doc_id, (term_count, score))| {
                self.documents.get(&doc_id).map(|doc| SearchResult {
                    document: doc.clone(),
                    score,
                    highlights: self.generate_highlights(&doc.content, &query_terms),
                    metadata: HashMap::new(),
                })
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply pagination
        let limit = query.limit.unwrap_or(10);
        let offset = query.offset.unwrap_or(0);

        let total = results.len();
        let paginated_results = results
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        let took_ms = start_time.elapsed().as_millis() as u64;

        let response = SearchResponse {
            results: paginated_results,
            total,
            query: query.query.clone(),
            took_ms,
            knowledge_base_id: "all".to_string(),
            pagination: Some(PaginationInfo {
                page: offset / limit + 1,
                per_page: limit,
                total_pages: (total + limit - 1) / limit,
                has_next: offset + limit < total,
                has_prev: offset > 0,
            }),
        };

        Ok(response)
    }

    /// Search within a specific knowledge base
    pub async fn search_knowledge_base(
        &self,
        kb_id: &str,
        query: SearchQuery,
    ) -> Result<SearchResponse, String> {
        if !self.knowledge_bases.contains_key(kb_id) {
            return Err("Knowledge base not found".to_string());
        }

        let start_time = std::time::Instant::now();

        // Filter documents by knowledge base
        let kb_docs: Vec<&KnowledgeDocument> = self.documents
            .values()
            .filter(|doc| doc.knowledge_base_id == kb_id)
            .collect();

        // Parse query into terms
        let query_terms: Vec<String> = self.extract_words(&query.query);

        if query_terms.is_empty() {
            return Err("Search query cannot be empty".to_string());
        }

        // Search within the filtered documents
        let mut results: Vec<SearchResult> = Vec::new();

        for doc in kb_docs {
            let mut score = 0.0;
            let mut term_matches = 0;

            // Search in content
            let content_words = self.extract_words(&doc.content);
            let title_words = self.extract_words(&doc.title);
            let summary_words = self.extract_words(&doc.summary.as_deref().unwrap_or(""));

            for term in &query_terms {
                if content_words.contains(&term) {
                    score += 2.0; // Content matches have higher weight
                    term_matches += 1;
                }
                if title_words.contains(&term) {
                    score += 3.0; // Title matches have highest weight
                    term_matches += 1;
                }
                if summary_words.contains(&term) {
                    score += 1.5; // Summary matches have medium weight
                    term_matches += 1;
                }
            }

            if term_matches > 0 {
                results.push(SearchResult {
                    document: doc.clone(),
                    score,
                    highlights: self.generate_highlights(&doc.content, &query_terms),
                    metadata: HashMap::new(),
                });
            }
        }

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply pagination
        let limit = query.limit.unwrap_or(10);
        let offset = query.offset.unwrap_or(0);

        let total = results.len();
        let paginated_results = results
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        let took_ms = start_time.elapsed().as_millis() as u64;

        let response = SearchResponse {
            results: paginated_results,
            total,
            query: query.query.clone(),
            took_ms,
            knowledge_base_id: kb_id.to_string(),
            pagination: Some(PaginationInfo {
                page: offset / limit + 1,
                per_page: limit,
                total_pages: (total + limit - 1) / limit,
                has_next: offset + limit < total,
                has_prev: offset > 0,
            }),
        };

        Ok(response)
    }

    /// Generate search highlights
    fn generate_highlights(&self, content: &str, query_terms: &[String]) -> Vec<String> {
        let mut highlights = Vec::new();

        for term in query_terms {
            if let Some(pos) = content.to_lowercase().find(&term.to_lowercase()) {
                let start = pos.saturating_sub(20);
                let end = (pos + term.len() + 20).min(content.len());
                let highlight = format!("...{}...", &content[start..end]);
                highlights.push(format!("\"{}\"", highlight));
            }
        }

        highlights
    }

    /// Get document by ID
    pub fn get_document(&self, doc_id: &str) -> Option<&KnowledgeDocument> {
        self.documents.get(doc_id)
    }

    /// List documents in a knowledge base
    pub fn list_documents(&self, kb_id: &str) -> Vec<&KnowledgeDocument> {
        self.documents
            .values()
            .filter(|doc| doc.knowledge_base_id == kb_id)
            .collect()
    }

    /// Get search statistics
    pub fn get_search_statistics(&self) -> HashMap<String, serde_json::Value> {
        let total_documents = self.documents.len();
        let total_terms = self.search_index.len();
        let total_knowledge_bases = self.knowledge_bases.len();

        let mut kb_stats = HashMap::new();
        for kb in self.knowledge_bases.values() {
            kb_stats.insert(
                kb.id.clone(),
                serde_json::json!({
                    "name": kb.name,
                    "document_count": kb.document_count,
                    "tags": kb.tags,
                }),
            );
        }

        HashMap::from([
            ("total_documents".to_string(), serde_json::Value::Number(total_documents.into())),
            ("total_terms".to_string(), serde_json::Value::Number(total_terms.into())),
            ("total_knowledge_bases".to_string(), serde_json::Value::Number(total_knowledge_bases.into())),
            ("knowledge_bases".to_string(), serde_json::to_value(&kb_stats).unwrap_or(serde_json::Value::Null)),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_search_creation() {
        let engine = KnowledgeSearchEngine::new();
        let kbs = engine.list_knowledge_bases();
        assert!(!kbs.is_empty());
        assert!(kbs.len() >= 2); // Should have at least tech and business knowledge bases
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let engine = KnowledgeSearchEngine::new();

        let query = SearchQuery {
            query: "rust programming".to_string(),
            limit: Some(5),
            offset: None,
            tags: None,
            filters: None,
            sort_by: None,
            order: None,
        };

        let response = engine.search(query).await.unwrap();
        assert!(!response.results.is_empty());
        assert_eq!(response.query, "rust programming");
    }

    #[tokio::test]
    async fn test_kb_specific_search() {
        let engine = KnowledgeSearchEngine::new();

        let query = SearchQuery {
            query: "onboarding process".to_string(),
            limit: Some(10),
            offset: None,
            tags: None,
            filters: None,
            sort_by: None,
            order: None,
        };

        let response = engine.search_knowledge_base("kb-business", query).await.unwrap();
        assert!(!response.results.is_empty());
        assert_eq!(response.knowledge_base_id, "kb-business");
    }
}