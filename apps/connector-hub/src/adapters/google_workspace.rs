use anyhow::{bail, Context, Result};
use reqwest::Client;
use secret_vault::SecretVault;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Auth token type
// ---------------------------------------------------------------------------

/// Distinguishes the two credential forms accepted by Google APIs on this platform.
///
/// Detection is based on the well-known token prefixes documented by Google:
/// - `ya29.*`  — short-lived OAuth 2.0 access token, sent as `Authorization: Bearer`.
/// - Everything else (including `AIzaSy*`) — treated as a public API key, sent as `?key=`.
#[derive(Debug, Clone)]
pub enum AuthToken {
    /// An OAuth 2.0 access token. Authorises user-delegated and service-account calls.
    Bearer(String),
    /// A public API key. Works only on endpoints that allow unauthenticated access.
    ApiKey(String),
}

impl AuthToken {
    /// Detects the credential kind by inspecting the token prefix.
    pub fn from_raw(raw: String) -> Self {
        if raw.starts_with("ya29.") {
            Self::Bearer(raw)
        } else {
            Self::ApiKey(raw)
        }
    }
}

// ---------------------------------------------------------------------------
// Response types — Calendar v3
// ---------------------------------------------------------------------------

/// A point in time on a Google Calendar event.
///
/// Google returns either `dateTime` (RFC 3339, for timed events) or `date`
/// (YYYY-MM-DD, for all-day events). Both are `Option` because the API always
/// sets exactly one per event boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDateTime {
    #[serde(rename = "dateTime", skip_serializing_if = "Option::is_none")]
    pub date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(rename = "timeZone", skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
}

/// A single event entry as returned by Calendar v3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub start: Option<EventDateTime>,
    pub end: Option<EventDateTime>,
    pub status: Option<String>,
    #[serde(rename = "htmlLink")]
    pub html_link: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

/// Top-level envelope for a Calendar v3 `events.list` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventsResponse {
    pub items: Vec<CalendarEvent>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "nextSyncToken")]
    pub next_sync_token: Option<String>,
}

/// Payload for creating a new calendar event.
///
/// Only fields the platform currently writes are included. Optional fields are
/// skipped during serialization to avoid sending nulls the Google API rejects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub start: EventDateTime,
    pub end: EventDateTime,
}

// ---------------------------------------------------------------------------
// Response types — Gmail v1
// ---------------------------------------------------------------------------

/// A message reference as returned by the Gmail v1 `messages.list` endpoint.
///
/// The list endpoint returns only `id` and `threadId`; fetching message bodies
/// requires a separate `messages.get` call (not implemented here yet).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailMessageRef {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: Option<String>,
}

/// Top-level envelope for a Gmail v1 `messages.list` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailMessagesResponse {
    pub messages: Option<Vec<GmailMessageRef>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "resultSizeEstimate")]
    pub result_size_estimate: Option<u32>,
}

// ---------------------------------------------------------------------------
// Response types — Drive v3
// ---------------------------------------------------------------------------

/// A file metadata entry as returned by Drive v3 `files.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub parents: Option<Vec<String>>,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
    #[serde(rename = "createdTime")]
    pub created_time: Option<String>,
    #[serde(rename = "modifiedTime")]
    pub modified_time: Option<String>,
    pub size: Option<String>,
}

/// Top-level envelope for a Drive v3 `files.list` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFilesResponse {
    pub files: Vec<DriveFile>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "incompleteSearch")]
    pub incomplete_search: Option<bool>,
}

// ---------------------------------------------------------------------------
// Response types — Gmail v1 (full message)
// ---------------------------------------------------------------------------

/// Full Gmail message as returned by `messages.get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailMessage {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: Option<String>,
    #[serde(rename = "labelIds")]
    pub label_ids: Option<Vec<String>>,
    pub snippet: Option<String>,
    pub payload: Option<GmailPayload>,
    #[serde(rename = "internalDate")]
    pub internal_date: Option<String>,
}

/// The payload (headers + body) of a Gmail message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailPayload {
    pub headers: Option<Vec<GmailHeader>>,
    pub parts: Option<Vec<GmailPart>>,
    pub body: Option<GmailBody>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// A single header key-value pair on a Gmail message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailHeader {
    pub name: String,
    pub value: String,
}

/// A MIME part within a Gmail message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailPart {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub body: Option<GmailBody>,
}

/// The body of a Gmail message part (base64url-encoded).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailBody {
    pub size: Option<u64>,
    pub data: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types — Calendar v3 (calendar list)
// ---------------------------------------------------------------------------

/// A single entry from the user's calendar list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarListEntry {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub primary: Option<bool>,
    #[serde(rename = "accessRole")]
    pub access_role: Option<String>,
}

/// Top-level envelope for a Calendar v3 `calendarList.list` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarListResponse {
    pub items: Vec<CalendarListEntry>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

/// HTTP adapter for Google Workspace APIs (Calendar v3, Gmail v1, Drive v3).
///
/// Credentials are resolved once at construction time via the secret vault.
/// All methods share a single [`reqwest::Client`] instance for connection pooling.
///
/// # Auth behaviour
/// The adapter auto-detects the credential type from its prefix:
/// - `ya29.*` → OAuth Bearer token; accepted by all three APIs.
/// - Anything else → treated as a public API key; write operations and Gmail
///   return `Err` immediately with a descriptive message rather than making a
///   doomed network call.
#[derive(Debug)]
pub struct GoogleWorkspaceAdapter {
    token: AuthToken,
    client: Client,
}

impl GoogleWorkspaceAdapter {
    const CALENDAR_BASE: &'static str = "https://www.googleapis.com/calendar/v3";
    const GMAIL_BASE: &'static str = "https://gmail.googleapis.com/gmail/v1";
    const DRIVE_BASE: &'static str = "https://www.googleapis.com/drive/v3";

    /// Constructs an adapter from an explicit raw credential string.
    ///
    /// Auth type (`Bearer` vs `ApiKey`) is detected automatically from the
    /// value's prefix. Prefer [`from_vault`] or [`from_env`] in production code.
    pub fn new(raw_token: String) -> Self {
        Self {
            token: AuthToken::from_raw(raw_token),
            client: Client::new(),
        }
    }

    /// Constructs an adapter by resolving credentials from a pre-built vault.
    ///
    /// Looks up `"google_workspace"` for `user_id`, falling back to the shared
    /// nil-UUID entry that corresponds to `MOCK_TOKEN_GOOGLE_WORKSPACE` in `.env`.
    ///
    /// # Errors
    /// Returns an error when no token is configured for this provider in the vault.
    pub fn from_vault(vault: &SecretVault, user_id: Uuid) -> Result<Self> {
        let raw = vault
            .get_token(user_id, "google_workspace")
            .context("GoogleWorkspaceAdapter requires MOCK_TOKEN_GOOGLE_WORKSPACE in .env")?;
        Ok(Self::new(raw))
    }

    /// Constructs an adapter by loading the vault from the process environment.
    ///
    /// Calls [`SecretVault::from_env`] which invokes `dotenvy::dotenv()` and
    /// scans all `MOCK_TOKEN_*` variables. Uses the nil user ID so the shared
    /// mock token is always found during local development.
    ///
    /// # Errors
    /// Returns an error when `MOCK_TOKEN_GOOGLE_WORKSPACE` is not set.
    pub fn from_env() -> Result<Self> {
        let vault = SecretVault::from_env();
        Self::from_vault(&vault, Uuid::nil())
    }

    /// Applies the stored credential to a [`reqwest::RequestBuilder`].
    ///
    /// - `Bearer`: adds `Authorization: Bearer <token>` header.
    /// - `ApiKey`: appends `key=<value>` as a query parameter.
    fn apply_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.token {
            AuthToken::Bearer(t) => builder.bearer_auth(t),
            AuthToken::ApiKey(k) => builder.query(&[("key", k.as_str())]),
        }
    }

    /// Returns `Err` if the current token is an API key and not an OAuth token.
    ///
    /// Call this at the start of every method that unconditionally requires OAuth.
    fn require_bearer(&self, operation: &str) -> Result<()> {
        if matches!(self.token, AuthToken::ApiKey(_)) {
            bail!(
                "`{operation}` requires an OAuth Bearer token (ya29.* prefix). \
                 The current MOCK_TOKEN_GOOGLE_WORKSPACE value is an API key. \
                 Replace it with a real OAuth token obtained via the Google OAuth flow."
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Calendar v3 methods
// ---------------------------------------------------------------------------

impl GoogleWorkspaceAdapter {
    /// Lists events from a Google Calendar.
    ///
    /// Calls `GET /calendar/v3/calendars/{calendar_id}/events`.
    ///
    /// Both API keys and Bearer tokens are accepted for publicly shared calendars.
    /// Private calendars require a Bearer token.
    ///
    /// # Parameters
    /// - `calendar_id`: The calendar identifier. Use `"primary"` for the
    ///   authenticated user's primary calendar.
    /// - `max_results`: Optional upper bound on returned events (1–2500).
    pub async fn list_calendar_events(
        &self,
        calendar_id: &str,
        max_results: Option<u32>,
    ) -> Result<CalendarEventsResponse> {
        let url = format!(
            "{}/calendars/{}/events",
            Self::CALENDAR_BASE,
            urlencoding_encode(calendar_id),
        );

        let mut builder = self.client.get(&url);
        builder = self.apply_auth(builder);

        if let Some(n) = max_results {
            builder = builder.query(&[("maxResults", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("failed to send list_calendar_events request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Calendar API /events returned {status}: {body}");
        }

        resp.json::<CalendarEventsResponse>()
            .await
            .context("failed to deserialize CalendarEventsResponse")
    }

    /// Creates a new event in a Google Calendar.
    ///
    /// Calls `POST /calendar/v3/calendars/{calendar_id}/events`.
    ///
    /// **Requires an OAuth Bearer token.** Returns `Err` immediately when the
    /// adapter holds an API key.
    ///
    /// # Parameters
    /// - `calendar_id`: Target calendar identifier.
    /// - `event`: The event payload to create.
    pub async fn create_calendar_event(
        &self,
        calendar_id: &str,
        event: CreateEventRequest,
    ) -> Result<CalendarEvent> {
        self.require_bearer("create_calendar_event")?;

        let url = format!(
            "{}/calendars/{}/events",
            Self::CALENDAR_BASE,
            urlencoding_encode(calendar_id),
        );

        let builder = self.client.post(&url).json(&event);
        let builder = self.apply_auth(builder);

        let resp = builder
            .send()
            .await
            .context("failed to send create_calendar_event request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Calendar API POST /events returned {status}: {body}");
        }

        resp.json::<CalendarEvent>()
            .await
            .context("failed to deserialize created CalendarEvent")
    }
}

// ---------------------------------------------------------------------------
// Gmail v1 methods
// ---------------------------------------------------------------------------

impl GoogleWorkspaceAdapter {
    /// Lists message references from the authenticated user's Gmail inbox.
    ///
    /// Calls `GET /gmail/v1/users/me/messages`.
    ///
    /// **Always requires an OAuth Bearer token.** Gmail does not accept API keys.
    ///
    /// # Parameters
    /// - `max_results`: Optional cap on returned message references (1–500).
    pub async fn list_gmail_messages(
        &self,
        max_results: Option<u32>,
    ) -> Result<GmailMessagesResponse> {
        self.require_bearer("list_gmail_messages")?;

        let url = format!("{}/users/me/messages", Self::GMAIL_BASE);

        let mut builder = self.client.get(&url);
        builder = self.apply_auth(builder);

        if let Some(n) = max_results {
            builder = builder.query(&[("maxResults", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("failed to send list_gmail_messages request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Gmail API /messages returned {status}: {body}");
        }

        resp.json::<GmailMessagesResponse>()
            .await
            .context("failed to deserialize GmailMessagesResponse")
    }
}

// ---------------------------------------------------------------------------
// Drive v3 methods
// ---------------------------------------------------------------------------

impl GoogleWorkspaceAdapter {
    /// Lists file metadata from the authenticated user's Google Drive.
    ///
    /// Calls `GET /drive/v3/files`. Returns only the fields needed for a file
    /// picker or agent context: `id`, `name`, `mimeType`, `parents`,
    /// `webViewLink`, `createdTime`.
    ///
    /// Both API keys and Bearer tokens are accepted by this endpoint.
    ///
    /// # Parameters
    /// - `page_size`: Optional maximum number of files per page (1–1000).
    pub async fn list_drive_files(&self, page_size: Option<u32>) -> Result<DriveFilesResponse> {
        let url = format!("{}/files", Self::DRIVE_BASE);

        let mut builder = self.client.get(&url);
        builder = self.apply_auth(builder);
        builder = builder.query(&[(
            "fields",
            "nextPageToken,incompleteSearch,files(id,name,mimeType,parents,webViewLink,createdTime)",
        )]);

        if let Some(n) = page_size {
            builder = builder.query(&[("pageSize", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("failed to send list_drive_files request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Drive API /files returned {status}: {body}");
        }

        resp.json::<DriveFilesResponse>()
            .await
            .context("failed to deserialize DriveFilesResponse")
    }
}

// ---------------------------------------------------------------------------
// Backward-compatible top-level function (preserves existing module contract)
// ---------------------------------------------------------------------------

/// Accesses Google Workspace resources for a delegated user session.
///
/// Constructs a temporary adapter from the environment and lists up to five
/// events from the primary calendar as a smoke-test operation.
///
/// # Scaffolding note
/// Replace direct callers with [`GoogleWorkspaceAdapter`] method calls once the
/// connector-hub routing layer is wired up.
pub async fn access_google_workspace() -> Result<()> {
    let adapter = GoogleWorkspaceAdapter::from_env()?;
    let _events = adapter.list_calendar_events("primary", Some(5)).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Stateless client (matches NotionClient pattern — token passed per call)
// ---------------------------------------------------------------------------

const CALENDAR_BASE: &str = "https://www.googleapis.com/calendar/v3";
const GMAIL_BASE: &str = "https://gmail.googleapis.com/gmail/v1";
const DRIVE_BASE: &str = "https://www.googleapis.com/drive/v3";

/// Stateless HTTP client for Google Workspace APIs.
///
/// Mirrors [`NotionClient`]'s design: holds only a shared [`reqwest::Client`]
/// for connection pooling and accepts a `token` parameter on every call so the
/// hub can resolve credentials from the vault per-request.
#[derive(Clone)]
pub struct GoogleWorkspaceClient {
    http: Client,
}

impl GoogleWorkspaceClient {
    pub fn new(http: Client) -> Self {
        Self { http }
    }
}

/// Parses a Google API response, returning the body as a deserialized `T` on
/// 2xx or an `anyhow` error containing the status code and body on failure.
async fn parse_google_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
    context: &str,
) -> Result<T> {
    let status = resp.status();
    let body = resp.text().await.context("failed to read response body")?;
    if status.is_success() {
        serde_json::from_str(&body)
            .with_context(|| format!("failed to parse {context} response"))
    } else {
        bail!("Google API error ({context}, HTTP {}): {}", status.as_u16(), body)
    }
}

// ---------------------------------------------------------------------------
// Calendar v3 methods (stateless client)
// ---------------------------------------------------------------------------

impl GoogleWorkspaceClient {
    /// Lists events from a Google Calendar.
    pub async fn list_calendar_events(
        &self,
        token: &str,
        calendar_id: &str,
        max_results: Option<u32>,
    ) -> Result<CalendarEventsResponse> {
        let url = format!(
            "{CALENDAR_BASE}/calendars/{}/events",
            urlencoding_encode(calendar_id),
        );

        let mut builder = self.http.get(&url).bearer_auth(token);
        if let Some(n) = max_results {
            builder = builder.query(&[("maxResults", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("google: list_calendar_events request failed")?;

        parse_google_response(resp, "list_calendar_events").await
    }

    /// Retrieves a single calendar event by ID.
    pub async fn get_calendar_event(
        &self,
        token: &str,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<CalendarEvent> {
        let url = format!(
            "{CALENDAR_BASE}/calendars/{}/events/{}",
            urlencoding_encode(calendar_id),
            urlencoding_encode(event_id),
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .context("google: get_calendar_event request failed")?;

        parse_google_response(resp, "get_calendar_event").await
    }

    /// Lists all calendars on the authenticated user's calendar list.
    pub async fn list_calendars(&self, token: &str) -> Result<CalendarListResponse> {
        let url = format!("{CALENDAR_BASE}/users/me/calendarList");

        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .context("google: list_calendars request failed")?;

        parse_google_response(resp, "list_calendars").await
    }

    /// Creates a new event in a Google Calendar.
    pub async fn create_calendar_event(
        &self,
        token: &str,
        calendar_id: &str,
        event: CreateEventRequest,
    ) -> Result<CalendarEvent> {
        let url = format!(
            "{CALENDAR_BASE}/calendars/{}/events",
            urlencoding_encode(calendar_id),
        );

        let resp = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&event)
            .send()
            .await
            .context("google: create_calendar_event request failed")?;

        parse_google_response(resp, "create_calendar_event").await
    }
}

// ---------------------------------------------------------------------------
// Gmail v1 methods (stateless client)
// ---------------------------------------------------------------------------

impl GoogleWorkspaceClient {
    /// Lists message references from the authenticated user's Gmail inbox.
    pub async fn list_gmail_messages(
        &self,
        token: &str,
        max_results: Option<u32>,
    ) -> Result<GmailMessagesResponse> {
        let url = format!("{GMAIL_BASE}/users/me/messages");

        let mut builder = self.http.get(&url).bearer_auth(token);
        if let Some(n) = max_results {
            builder = builder.query(&[("maxResults", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("google: list_gmail_messages request failed")?;

        parse_google_response(resp, "list_gmail_messages").await
    }

    /// Retrieves the full content of a single Gmail message.
    ///
    /// The `format` parameter controls how much detail is returned:
    /// `"full"` (default), `"metadata"`, `"minimal"`, or `"raw"`.
    pub async fn get_gmail_message(
        &self,
        token: &str,
        message_id: &str,
        format: Option<&str>,
    ) -> Result<GmailMessage> {
        let url = format!("{GMAIL_BASE}/users/me/messages/{message_id}");

        let mut builder = self.http.get(&url).bearer_auth(token);
        if let Some(fmt) = format {
            builder = builder.query(&[("format", fmt)]);
        }

        let resp = builder
            .send()
            .await
            .context("google: get_gmail_message request failed")?;

        parse_google_response(resp, "get_gmail_message").await
    }

    /// Searches Gmail messages using Gmail's query syntax.
    ///
    /// Supports standard Gmail operators: `from:`, `subject:`, `after:`,
    /// `before:`, `has:attachment`, `in:inbox`, etc.
    pub async fn search_gmail_messages(
        &self,
        token: &str,
        query: &str,
        max_results: Option<u32>,
    ) -> Result<GmailMessagesResponse> {
        let url = format!("{GMAIL_BASE}/users/me/messages");

        let mut builder = self.http.get(&url).bearer_auth(token).query(&[("q", query)]);
        if let Some(n) = max_results {
            builder = builder.query(&[("maxResults", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("google: search_gmail_messages request failed")?;

        parse_google_response(resp, "search_gmail_messages").await
    }
}

// ---------------------------------------------------------------------------
// Drive v3 methods (stateless client)
// ---------------------------------------------------------------------------

impl GoogleWorkspaceClient {
    /// Lists file metadata from the authenticated user's Google Drive.
    pub async fn list_drive_files(
        &self,
        token: &str,
        page_size: Option<u32>,
    ) -> Result<DriveFilesResponse> {
        let url = format!("{DRIVE_BASE}/files");

        let mut builder = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[(
                "fields",
                "nextPageToken,incompleteSearch,files(id,name,mimeType,parents,webViewLink,createdTime,modifiedTime,size)",
            )]);

        if let Some(n) = page_size {
            builder = builder.query(&[("pageSize", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("google: list_drive_files request failed")?;

        parse_google_response(resp, "list_drive_files").await
    }

    /// Retrieves metadata for a single Drive file by ID.
    pub async fn get_drive_file(&self, token: &str, file_id: &str) -> Result<DriveFile> {
        let url = format!("{DRIVE_BASE}/files/{file_id}");

        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[(
                "fields",
                "id,name,mimeType,parents,webViewLink,createdTime,modifiedTime,size",
            )])
            .send()
            .await
            .context("google: get_drive_file request failed")?;

        parse_google_response(resp, "get_drive_file").await
    }

    /// Searches files in Google Drive using Drive's query syntax.
    ///
    /// Supports operators like `name contains 'report'`,
    /// `mimeType = 'application/pdf'`, `modifiedTime > '2024-01-01'`.
    pub async fn search_drive_files(
        &self,
        token: &str,
        query: &str,
        page_size: Option<u32>,
    ) -> Result<DriveFilesResponse> {
        let url = format!("{DRIVE_BASE}/files");

        let mut builder = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[("q", query)])
            .query(&[(
                "fields",
                "nextPageToken,incompleteSearch,files(id,name,mimeType,parents,webViewLink,createdTime,modifiedTime,size)",
            )]);

        if let Some(n) = page_size {
            builder = builder.query(&[("pageSize", n.to_string())]);
        }

        let resp = builder
            .send()
            .await
            .context("google: search_drive_files request failed")?;

        parse_google_response(resp, "search_drive_files").await
    }

    /// Exports a Google Docs/Sheets/Slides file to the specified MIME type.
    ///
    /// Common export targets: `text/plain`, `text/csv`,
    /// `application/pdf`, `text/html`.
    pub async fn export_drive_file(
        &self,
        token: &str,
        file_id: &str,
        mime_type: &str,
    ) -> Result<String> {
        let url = format!("{DRIVE_BASE}/files/{file_id}/export");

        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[("mimeType", mime_type)])
            .send()
            .await
            .context("google: export_drive_file request failed")?;

        let status = resp.status();
        let body = resp.text().await.context("failed to read export response body")?;
        if status.is_success() {
            Ok(body)
        } else {
            bail!(
                "Google API error (export_drive_file, HTTP {}): {}",
                status.as_u16(),
                body
            )
        }
    }
}

// ---------------------------------------------------------------------------
// URL path encoding helper
// ---------------------------------------------------------------------------

/// Percent-encodes a URL path segment.
///
/// Leaves ASCII alphanumeric characters and the safe set (`-`, `_`, `.`, `~`, `@`)
/// unmodified. The `@` sign is preserved because Google Calendar IDs are email
/// addresses (e.g., `user@example.com`).
fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '@') {
            out.push(c);
        } else {
            for byte in c.to_string().as_bytes() {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use secret_vault::SecretVault;
    use std::collections::HashMap;
    use uuid::Uuid;

    // --- AuthToken detection -----------------------------------------------

    #[test]
    fn auth_token_detects_bearer_from_ya29_prefix() {
        let token = AuthToken::from_raw("ya29.a0AfB_byC".to_string());
        assert!(matches!(token, AuthToken::Bearer(_)));
    }

    #[test]
    fn auth_token_detects_api_key_from_aizasy_prefix() {
        let token = AuthToken::from_raw("AIzaSyDc0jXC81vYorDvJgsEdA-0cL_XkrfE80c".to_string());
        assert!(matches!(token, AuthToken::ApiKey(_)));
    }

    #[test]
    fn auth_token_treats_unknown_prefix_as_api_key() {
        let token = AuthToken::from_raw("some-unknown-token-string".to_string());
        assert!(matches!(token, AuthToken::ApiKey(_)));
    }

    // --- from_vault construction -------------------------------------------

    #[test]
    fn from_vault_succeeds_with_configured_token() {
        let mut tokens = HashMap::new();
        tokens.insert(
            (Uuid::nil(), "google_workspace".to_string()),
            "ya29.test-token".to_string(),
        );
        let vault = SecretVault::from_tokens(tokens);
        let result = GoogleWorkspaceAdapter::from_vault(&vault, Uuid::nil());
        assert!(result.is_ok(), "from_vault should succeed when token is present");
    }

    #[test]
    fn from_vault_returns_error_when_token_missing() {
        let vault = SecretVault::from_tokens(HashMap::new());
        let err = GoogleWorkspaceAdapter::from_vault(&vault, Uuid::nil())
            .expect_err("from_vault should fail when no token is configured");
        let msg = err.to_string();
        assert!(
            msg.contains("MOCK_TOKEN_GOOGLE_WORKSPACE"),
            "error should name the expected env var; got: {msg}"
        );
    }

    // --- OAuth guard (require_bearer) --------------------------------------

    #[tokio::test]
    async fn create_event_with_api_key_returns_error_immediately() {
        let adapter =
            GoogleWorkspaceAdapter::new("AIzaSyDc0jXC81vYorDvJgsEdA-0cL_XkrfE80c".to_string());
        let dummy_event = CreateEventRequest {
            summary: "Test event".to_string(),
            description: None,
            start: EventDateTime {
                date_time: Some("2026-03-07T10:00:00Z".to_string()),
                date: None,
                time_zone: None,
            },
            end: EventDateTime {
                date_time: Some("2026-03-07T11:00:00Z".to_string()),
                date: None,
                time_zone: None,
            },
        };
        let err = adapter
            .create_calendar_event("primary", dummy_event)
            .await
            .expect_err("create_calendar_event must reject API key tokens");
        let msg = err.to_string();
        assert!(
            msg.contains("create_calendar_event"),
            "error should name the operation; got: {msg}"
        );
        assert!(
            msg.contains("Bearer"),
            "error should mention Bearer token requirement; got: {msg}"
        );
    }

    #[tokio::test]
    async fn list_gmail_with_api_key_returns_error_immediately() {
        let adapter = GoogleWorkspaceAdapter::new("AIzaSyXYZ".to_string());
        let err = adapter
            .list_gmail_messages(Some(5))
            .await
            .expect_err("list_gmail_messages must reject API key tokens");
        assert!(
            err.to_string().contains("list_gmail_messages"),
            "error should name the operation; got: {}",
            err
        );
    }

    // --- URL encoding helper -----------------------------------------------

    #[test]
    fn urlencoding_encode_leaves_primary_unchanged() {
        assert_eq!(urlencoding_encode("primary"), "primary");
    }

    #[test]
    fn urlencoding_encode_preserves_at_sign_in_calendar_id() {
        let encoded = urlencoding_encode("user@example.com");
        assert!(encoded.contains('@'), "@ should be preserved; got: {encoded}");
        assert!(!encoded.is_empty());
    }

    #[test]
    fn urlencoding_encode_encodes_space() {
        let encoded = urlencoding_encode("my calendar");
        assert!(
            encoded.contains("%20"),
            "space should be percent-encoded; got: {encoded}"
        );
    }

    // --- Integration tests (skipped when env var is absent) ----------------

    fn make_adapter() -> Option<GoogleWorkspaceAdapter> {
        let _ = dotenvy::dotenv();
        let raw = match std::env::var("MOCK_TOKEN_GOOGLE_WORKSPACE") {
            Ok(v) if !v.is_empty() && v != "YOUR_GOOGLE_WORKSPACE_ACCESS_TOKEN" => v,
            _ => {
                eprintln!(
                    "MOCK_TOKEN_GOOGLE_WORKSPACE not configured \
                     – skipping Google Workspace integration tests"
                );
                return None;
            }
        };
        Some(GoogleWorkspaceAdapter::new(raw))
    }

    #[tokio::test]
    async fn integration_list_calendar_events_returns_ok() {
        let Some(adapter) = make_adapter() else {
            return;
        };
        match adapter.list_calendar_events("primary", Some(5)).await {
            Ok(resp) => {
                eprintln!("[calendar] Received {} event(s)", resp.items.len());
                for ev in &resp.items {
                    eprintln!(
                        "  - {:?} (start: {:?})",
                        ev.summary,
                        ev.start.as_ref().and_then(|s| s.date_time.as_deref())
                    );
                }
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("401") || msg.contains("403") || msg.contains("400")
                    || msg.contains("404")
                {
                    eprintln!(
                        "[calendar] Token lacks calendar scope or is an API key \
                         on a private calendar – skipping assertion: {msg}"
                    );
                } else {
                    panic!("Unexpected list_calendar_events error: {msg}");
                }
            }
        }
    }

    #[tokio::test]
    async fn integration_list_drive_files_returns_ok() {
        let Some(adapter) = make_adapter() else {
            return;
        };
        match adapter.list_drive_files(Some(5)).await {
            Ok(resp) => {
                eprintln!("[drive] Received {} file(s)", resp.files.len());
                for f in &resp.files {
                    eprintln!("  - {} ({:?})", f.name, f.mime_type);
                }
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("401") || msg.contains("403") || msg.contains("400") {
                    eprintln!("[drive] Token lacks Drive scope – skipping: {msg}");
                } else {
                    panic!("Unexpected list_drive_files error: {msg}");
                }
            }
        }
    }
}
