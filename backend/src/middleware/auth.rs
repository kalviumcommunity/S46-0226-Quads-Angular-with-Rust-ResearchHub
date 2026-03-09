use crate::{
    app_state::AppState,
    auth::jwt,
    error::AppError,
    models::user::{Role, User},
};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: Role,
    pub group_id: Option<Uuid>,
    pub institution_id: Option<Uuid>,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(&request)?;
    let claims = jwt::decode_token(token, &state.config.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, full_name, password_hash, role, institution_id, group_id, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    request.extensions_mut().insert(AuthenticatedUser {
        id: user.id,
        role: user.role,
        group_id: user.group_id,
        institution_id: user.institution_id,
    });

    Ok(next.run(request).await)
}

fn extract_bearer_token(request: &Request) -> Result<&str, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)
}
