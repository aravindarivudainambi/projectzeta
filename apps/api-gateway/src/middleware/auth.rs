use std::str;

use anyhow::Result;
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Typed request extension representing the authenticated user identifier.
#[derive(Debug, Clone, Copy)]
pub struct UserId(pub Uuid);

/// Typed request extension representing the authenticated tenant identifier.
#[derive(Debug, Clone, Copy)]
pub struct TenantId(pub Uuid);

#[derive(Debug, Deserialize)]
struct TokenClaims {
    sub: String,
    tenant_id: String,
}

/// Validates and extracts user context from an incoming authorization token.
///
/// This middleware expects an `Authorization: Bearer <token>` header where the
/// token uses JWT structure. The middleware decodes the middle payload segment
/// using URL-safe base64, parses JSON claims, extracts `sub` as the user ID and
/// `tenant_id` as the tenant ID, and stores both values in request extensions.
///
/// If the header is missing, malformed, or the payload cannot be decoded into
/// valid UUID claims, this function returns an unauthorized response body:
/// `{"error":"unauthorized"}` with HTTP status `401`.
pub async fn auth_middleware(mut request: Request, next: Next) -> Response {
    let maybe_auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let Some(auth_header) = maybe_auth_header else {
        return unauthorized_response();
    };

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return unauthorized_response();
    };

    let Ok((user_id, tenant_id)) = decode_bearer_claims(token) else {
        return unauthorized_response();
    };

    request.extensions_mut().insert(UserId(user_id));
    request.extensions_mut().insert(TenantId(tenant_id));

    next.run(request).await
}

/// Decodes bearer token claims from the JWT payload and returns typed IDs.
///
/// This helper only performs payload decoding and claim parsing for scaffold
/// middleware. Signature verification and issuer/audience checks are expected
/// to be added when auth-service token contracts are finalized.
pub fn decode_bearer_claims(token: &str) -> Result<(Uuid, Uuid)> {
    let mut parts = token.split('.');
    let _header_segment = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing header"))?;
    let payload_segment = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing payload"))?;

    let payload_bytes = URL_SAFE_NO_PAD.decode(payload_segment)?;
    let payload_json = str::from_utf8(&payload_bytes)?;
    let claims: TokenClaims = serde_json::from_str(payload_json)?;

    let user_id = Uuid::parse_str(&claims.sub)?;
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    Ok((user_id, tenant_id))
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "unauthorized" })),
    )
        .into_response()
}
