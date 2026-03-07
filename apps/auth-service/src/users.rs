use argon2::{self, Config as ArgonConfig};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// JSON payload accepted by `POST /auth/register`.
///
/// This contract intentionally keeps the registration surface minimal so the auth
/// boundary can evolve independently from profile or tenant onboarding details.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// JSON response returned by successful registration.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub email: String,
}

/// Persists a newly registered user with an Argon2 password hash.
///
/// The handler performs the following workflow:
/// 1. Resolves a default tenant used by this scaffold service.
/// 2. Generates a cryptographically secure random salt.
/// 3. Hashes the plaintext password with `argon2::hash_encoded`.
/// 4. Inserts the user record into PostgreSQL.
///
/// When the email already exists, this endpoint returns `409 Conflict` and does
/// not alter the original user row.
pub async fn register_user(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_default_tenant(&pool).await {
        Ok(value) => value,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let mut salt = [0_u8; 32];
    OsRng.fill_bytes(&mut salt);

    let hash =
        match argon2::hash_encoded(payload.password.as_bytes(), &salt, &ArgonConfig::default()) {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

    let result = sqlx::query_as::<_, (Uuid,)>(
        "INSERT INTO users (tenant_id, email, password) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(tenant_id)
    .bind(payload.email.clone())
    .bind(hash)
    .fetch_one(&pool)
    .await;

    match result {
        Ok((user_id,)) => (
            StatusCode::CREATED,
            Json(RegisterResponse {
                user_id,
                email: payload.email,
            }),
        )
            .into_response(),
        Err(sqlx::Error::Database(db_error)) => {
            if is_unique_violation(db_error.as_ref()) {
                StatusCode::CONFLICT.into_response()
            } else {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Resolves a tenant identifier used for scaffold-level auth registration flows.
///
/// This helper avoids hardcoding a UUID in source code and ensures that users
/// created through local development endpoints remain explicitly tenant-scoped.
async fn resolve_default_tenant(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    if let Some((tenant_id,)) = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM tenants LIMIT 1")
        .fetch_optional(pool)
        .await?
    {
        return Ok(tenant_id);
    }

    let (tenant_id,) = sqlx::query_as::<_, (Uuid,)>(
        "INSERT INTO tenants (name) VALUES ('default-tenant') RETURNING id",
    )
    .fetch_one(pool)
    .await?;

    Ok(tenant_id)
}

fn is_unique_violation(db_error: &dyn sqlx::error::DatabaseError) -> bool {
    matches!(db_error.code().as_deref(), Some("23505"))
}
