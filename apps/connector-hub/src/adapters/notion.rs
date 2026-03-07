use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

const NOTION_API_BASE: &str = "https://api.notion.com/v1";
const NOTION_VERSION: &str = "2022-06-28";

/// HTTP client for the Notion API.
///
/// Holds a shared `reqwest::Client` for connection pooling but does NOT
/// store tokens. Every method accepts a `token` parameter so the caller
/// controls credential lifecycle via the token vault.
#[derive(Clone)]
pub struct NotionClient {
    http: Client,
}

impl NotionClient {
    pub fn new(http: Client) -> Self {
        Self { http }
    }

    // --- Databases --------------------------------------------------------

    /// Queries a Notion database with optional filter and sort criteria.
    pub async fn query_database(
        &self,
        token: &str,
        database_id: &str,
        filter: Option<Value>,
        sorts: Option<Value>,
    ) -> Result<Value> {
        let mut body = serde_json::Map::new();
        if let Some(f) = filter {
            body.insert("filter".to_string(), f);
        }
        if let Some(s) = sorts {
            body.insert("sorts".to_string(), s);
        }

        let resp = self
            .http
            .post(format!("{NOTION_API_BASE}/databases/{database_id}/query"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .json(&body)
            .send()
            .await
            .context("notion: query_database request failed")?;

        parse_response(resp).await
    }

    // --- Pages ------------------------------------------------------------

    /// Creates a new page in Notion.
    ///
    /// The `body` must include `parent` and `properties` per the Notion API spec.
    pub async fn create_page(&self, token: &str, body: Value) -> Result<Value> {
        let resp = self
            .http
            .post(format!("{NOTION_API_BASE}/pages"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .json(&body)
            .send()
            .await
            .context("notion: create_page request failed")?;

        parse_response(resp).await
    }

    /// Retrieves a page by ID.
    pub async fn retrieve_page(&self, token: &str, page_id: &str) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{NOTION_API_BASE}/pages/{page_id}"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .context("notion: retrieve_page request failed")?;

        parse_response(resp).await
    }

    /// Updates page properties.
    pub async fn update_page(
        &self,
        token: &str,
        page_id: &str,
        properties: Value,
    ) -> Result<Value> {
        let resp = self
            .http
            .patch(format!("{NOTION_API_BASE}/pages/{page_id}"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .json(&properties)
            .send()
            .await
            .context("notion: update_page request failed")?;

        parse_response(resp).await
    }

    // --- Search -----------------------------------------------------------

    /// Searches across pages and databases in the workspace.
    pub async fn search(
        &self,
        token: &str,
        query: Option<&str>,
        filter_object: Option<&str>,
    ) -> Result<Value> {
        let mut body = serde_json::Map::new();
        if let Some(q) = query {
            body.insert("query".to_string(), Value::String(q.to_string()));
        }
        if let Some(obj_type) = filter_object {
            body.insert(
                "filter".to_string(),
                serde_json::json!({
                    "value": obj_type,
                    "property": "object"
                }),
            );
        }

        let resp = self
            .http
            .post(format!("{NOTION_API_BASE}/search"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .json(&body)
            .send()
            .await
            .context("notion: search request failed")?;

        parse_response(resp).await
    }

    // --- Blocks -----------------------------------------------------------

    /// Retrieves a single block by ID.
    pub async fn retrieve_block(&self, token: &str, block_id: &str) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{NOTION_API_BASE}/blocks/{block_id}"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .context("notion: retrieve_block request failed")?;

        parse_response(resp).await
    }

    /// Lists all children blocks of a given block.
    pub async fn get_block_children(&self, token: &str, block_id: &str) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{NOTION_API_BASE}/blocks/{block_id}/children"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .context("notion: get_block_children request failed")?;

        parse_response(resp).await
    }

    /// Appends new children blocks to an existing block.
    pub async fn append_block_children(
        &self,
        token: &str,
        block_id: &str,
        children: Value,
    ) -> Result<Value> {
        let body = serde_json::json!({ "children": children });

        let resp = self
            .http
            .patch(format!("{NOTION_API_BASE}/blocks/{block_id}/children"))
            .bearer_auth(token)
            .header("Notion-Version", NOTION_VERSION)
            .json(&body)
            .send()
            .await
            .context("notion: append_block_children request failed")?;

        parse_response(resp).await
    }
}

/// Parses a Notion API response, returning the body as JSON on 2xx or an
/// `anyhow` error containing the status code and error body on failure.
async fn parse_response(resp: reqwest::Response) -> Result<Value> {
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;

    if status.is_success() {
        serde_json::from_str(&body).context("failed to parse Notion success response")
    } else {
        anyhow::bail!("Notion API error (HTTP {}): {}", status.as_u16(), body)
    }
}
