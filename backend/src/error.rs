use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("db error")]
    Db(#[from] sqlx::Error),
    #[error("token error")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("internal error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Db(_) | AppError::Jwt(_) | AppError::Internal => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = ErrorBody {
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}
