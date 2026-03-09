use crate::{
    app_state::AppState,
    auth::jwt,
    error::AppError,
    models::user::{RegisterUserInput, Role, User},
};
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub role: Role,
    pub institution_id: Option<uuid::Uuid>,
    pub group_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUser,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserInput>,
) -> Result<impl IntoResponse, AppError> {
    let role = payload.role.unwrap_or(Role::Student);

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, full_name, password_hash, role, institution_id, group_id)
         VALUES ($1, $2, $3, $4::user_role, $5, $6)
         RETURNING id, email, full_name, password_hash, role, institution_id, group_id, created_at, updated_at",
    )
    .bind(payload.email)
    .bind(payload.full_name)
    .bind(payload.password)
    .bind(format_role(&role))
    .bind(payload.institution_id)
    .bind(payload.group_id)
    .fetch_one(&state.db_pool)
    .await?;

    let token = jwt::create_token(user.id, &format_role(&user.role), &state.config.jwt_secret)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: map_user(user),
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, full_name, password_hash, role, institution_id, group_id, created_at, updated_at
         FROM users WHERE email = $1",
    )
    .bind(payload.email)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if user.password_hash != payload.password {
        return Err(AppError::Unauthorized);
    }

    let token = jwt::create_token(user.id, &format_role(&user.role), &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: map_user(user),
    }))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let token = extract_bearer_token(&headers)?;
    let claims = jwt::decode_token(token, &state.config.jwt_secret)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, full_name, password_hash, role, institution_id, group_id, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(
        uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::BadRequest("invalid subject".to_string()))?,
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(map_user(user)))
}

fn format_role(role: &Role) -> String {
    match role {
        Role::Faculty => "faculty".to_string(),
        Role::Student => "student".to_string(),
        Role::Admin => "admin".to_string(),
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let auth = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    auth.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)
}

fn map_user(user: User) -> AuthUser {
    AuthUser {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        role: user.role,
        institution_id: user.institution_id,
        group_id: user.group_id,
    }
}
