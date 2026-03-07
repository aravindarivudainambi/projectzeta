/// Represents high-level application errors that the gateway may map to HTTP responses.
#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    RateLimited,
    Internal,
}

/// Returns a placeholder response code for a high-level application error.
///
/// Replace this with an `IntoResponse` implementation once actual handlers are implemented.
pub fn status_code_for_error(error: &AppError) -> u16 {
    match error {
        AppError::Unauthorized => 401,
        AppError::RateLimited => 429,
        AppError::Internal => 500,
    }
}
