use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_handler, tool_router,
    transport::stdio,
    ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchSessionsRequest {
    #[schemars(description = "Query string to search for in session chunks")]
    pub query: String,
    #[schemars(description = "Optional session ID to filter results")]
    pub session_id: Option<String>,
    #[schemars(description = "Optional source filter: message, todo, error, decision, file_change, summary")]
    pub source: Option<String>,
    #[schemars(description = "Maximum results to return (default: 10)")]
    pub top_k: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchPromptsRequest {
    #[schemars(description = "Query string to search for in prompt documents")]
    pub query: String,
    #[schemars(description = "Optional library ID to filter results")]
    pub library_id: Option<String>,
    #[schemars(description = "Optional tags to filter results")]
    pub tags: Option<Vec<String>>,
    #[schemars(description = "Maximum results to return (default: 10)")]
    pub top_k: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CrossLibrarySearchRequest {
    #[schemars(description = "Query string to search across sessions and prompts")]
    pub query: String,
    #[schemars(description = "Optional session ID to filter session results")]
    pub session_id: Option<String>,
    #[schemars(description = "Optional library ID to filter prompt results")]
    pub library_id: Option<String>,
    #[schemars(description = "Maximum results per source (default: 5)")]
    pub top_k_per_source: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndexSessionChunkRequest {
    #[schemars(description = "Unique chunk ID")]
    pub chunk_id: String,
    #[schemars(description = "Session ID this chunk belongs to")]
    pub session_id: String,
    #[schemars(description = "Text content of the chunk")]
    pub text: String,
    #[schemars(description = "Source type: message, todo, error, decision, file_change, summary")]
    pub source: String,
    #[schemars(description = "Optional metadata dictionary")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndexPromptRequest {
    #[schemars(description = "Unique prompt ID")]
    pub prompt_id: String,
    #[schemars(description = "Prompt text content")]
    pub text: String,
    #[schemars(description = "Optional title")]
    pub title: Option<String>,
    #[schemars(description = "Optional description")]
    pub description: Option<String>,
    #[schemars(description = "Optional tags")]
    pub tags: Option<Vec<String>>,
    #[schemars(description = "Optional file path")]
    pub file_path: Option<String>,
    #[schemars(description = "Optional library ID (default: default)")]
    pub library_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderPromptRequest {
    #[schemars(description = "Template ID (prompt_id) to render")]
    pub template_id: String,
    #[schemars(description = "Variables to substitute in the template")]
    pub variables: std::collections::HashMap<String, String>,
}

#[derive(Clone)]
pub struct MemoryServer {
    client: reqwest::Client,
    base_url: String,
}

impl MemoryServer {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url,
        }
    }

    async fn post_json(
        &self,
        path: &str,
        body: impl Serialize,
    ) -> Result<serde_json::Value, rmcp::ErrorData> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(format!("HTTP request failed: {}", e), None))?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(format!("JSON parse failed: {}", e), None))?;

        if !status.is_success() {
            return Err(rmcp::ErrorData::internal_error(
                format!("Backend returned {}: {}", status, json),
                None,
            ));
        }

        Ok(json)
    }

    async fn get_json(&self, path: &str) -> Result<serde_json::Value, rmcp::ErrorData> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(format!("HTTP request failed: {}", e), None))?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(format!("JSON parse failed: {}", e), None))?;

        if !status.is_success() {
            return Err(rmcp::ErrorData::internal_error(
                format!("Backend returned {}: {}", status, json),
                None,
            ));
        }

        Ok(json)
    }

    fn format_json_result(value: serde_json::Value) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = serde_json::to_string_pretty(&value)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialize failed: {}", e), None))?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

#[tool_router]
impl MemoryServer {
    #[tool(name = "search_sessions", description = "Semantic search across indexed session chunks. Find relevant messages, errors, todos, decisions, and file changes from past conversations.")]
    async fn search_sessions(
        &self,
        req: Parameters<SearchSessionsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let req = req.0;
        let body = serde_json::json!({
            "query": req.query,
            "session_id": req.session_id,
            "source": req.source,
            "top_k": req.top_k.unwrap_or(10),
        });

        let result = self.post_json("/sessions/query", body).await?;
        Self::format_json_result(result)
    }

    #[tool(name = "search_prompts", description = "Semantic search across indexed prompt libraries. Find relevant prompts, templates, and documentation.")]
    async fn search_prompts(
        &self,
        req: Parameters<SearchPromptsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let req = req.0;
        let body = serde_json::json!({
            "query": req.query,
            "library_id": req.library_id,
            "tags": req.tags,
            "top_k": req.top_k.unwrap_or(10),
        });

        let result = self.post_json("/prompts/query", body).await?;
        Self::format_json_result(result)
    }

    #[tool(name = "cross_library_search", description = "Search across both sessions and prompts simultaneously. Returns top results from each source.")]
    async fn cross_library_search(
        &self,
        req: Parameters<CrossLibrarySearchRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let req = req.0;
        let body = serde_json::json!({
            "query": req.query,
            "session_id": req.session_id,
            "library_id": req.library_id,
            "top_k_per_source": req.top_k_per_source.unwrap_or(5),
        });

        let result = self.post_json("/search", body).await?;
        Self::format_json_result(result)
    }

    #[tool(name = "index_session_chunk", description = "Index a single session chunk into the vector store for semantic search. Use for real-time session indexing.")]
    async fn index_session_chunk(
        &self,
        req: Parameters<IndexSessionChunkRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let req = req.0;
        let body = serde_json::json!({
            "chunk_id": req.chunk_id,
            "session_id": req.session_id,
            "text": req.text,
            "source": req.source,
            "metadata": req.metadata.unwrap_or(serde_json::json!({})),
        });

        let result = self.post_json("/sessions/chunk", body).await?;
        Self::format_json_result(result)
    }

    #[tool(name = "index_prompt", description = "Index a single prompt document into the vector store. The prompt becomes searchable via semantic search.")]
    async fn index_prompt(
        &self,
        req: Parameters<IndexPromptRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let req = req.0;
        let body = serde_json::json!({
            "library_id": req.library_id.unwrap_or_else(|| "default".to_string()),
            "documents": [{
                "prompt_id": req.prompt_id,
                "text": req.text,
                "title": req.title,
                "description": req.description,
                "tags": req.tags.unwrap_or_default(),
                "file_path": req.file_path,
            }],
        });

        let result = self.post_json("/prompts/index", body).await?;
        Self::format_json_result(result)
    }

    #[tool(name = "render_prompt", description = "Render a prompt template with variable substitution. Uses Python string.Template.safe_substitute semantics.")]
    async fn render_prompt(
        &self,
        req: Parameters<RenderPromptRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let req = req.0;
        let body = serde_json::json!({
            "template_id": req.template_id,
            "variables": req.variables,
        });

        let result = self.post_json("/prompts/render", body).await?;
        Self::format_json_result(result)
    }

    #[tool(name = "list_sessions", description = "List all session IDs that have been indexed in the vector store.")]
    async fn list_sessions(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = self.get_json("/sessions/indexed").await?;
        Self::format_json_result(result)
    }

    #[tool(name = "list_libraries", description = "List all prompt library IDs that have been indexed.")]
    async fn list_libraries(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = self.get_json("/prompts/libraries").await?;
        Self::format_json_result(result)
    }
}

#[tool_handler(
    name = "rag-params-mcp",
    instructions = "Semantic memory MCP server for rag-params-finder. Provides tools to search, index, and render sessions and prompts."
)]
impl rmcp::ServerHandler for MemoryServer {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse()?),
        )
        .init();

    let base_url = std::env::var("RAG_PARAMS_API_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string());

    tracing::info!("Starting rag-params-mcp server, backend: {}", base_url);

    let service = MemoryServer::new(base_url)
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("Error starting server: {e}"))?;

    service.waiting().await?;
    Ok(())
}
