use crate::{
    app_state::AppState,
    error::AppError,
    middleware::auth::AuthenticatedUser,
    models::comment::{Comment, CreateCommentInput},
    models::research::ResearchItem,
    models::user::Role,
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_comment))
        .route("/research/:research_item_id", get(list_comments))
        .route("/:id", delete(delete_comment))
}

pub async fn create_comment(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateCommentInput>,
) -> Result<impl IntoResponse, AppError> {
    ensure_can_access_item(&state, &user, payload.research_item_id).await?;

    let comment = sqlx::query_as::<_, Comment>(
        "INSERT INTO comments (research_item_id, author_id, content)
         VALUES ($1, $2, $3)
         RETURNING id, research_item_id, author_id, content, created_at, updated_at",
    )
    .bind(payload.research_item_id)
    .bind(user.id)
    .bind(payload.content)
    .fetch_one(&state.db_pool)
    .await?;

    Ok((StatusCode::CREATED, Json(comment)))
}

pub async fn list_comments(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(research_item_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ensure_can_access_item(&state, &user, research_item_id).await?;

    let comments = sqlx::query_as::<_, Comment>(
        "SELECT id, research_item_id, author_id, content, created_at, updated_at
         FROM comments
         WHERE research_item_id = $1
         ORDER BY created_at ASC",
    )
    .bind(research_item_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(Json(comments))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let comment = sqlx::query_as::<_, Comment>(
        "SELECT id, research_item_id, author_id, content, created_at, updated_at
         FROM comments
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if user.role != Role::Admin && comment.author_id != user.id {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn ensure_can_access_item(
    state: &AppState,
    user: &AuthenticatedUser,
    research_item_id: Uuid,
) -> Result<(), AppError> {
    let item = sqlx::query_as::<_, ResearchItem>(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id, created_at, updated_at
         FROM research_items
         WHERE id = $1",
    )
    .bind(research_item_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if user.role == Role::Admin || item.owner_id == user.id {
        return Ok(());
    }

    let allowed = match item.visibility {
        crate::models::research::ResearchVisibility::Public => true,
        crate::models::research::ResearchVisibility::Institution => {
            item.institution_id == user.institution_id
        }
        crate::models::research::ResearchVisibility::Group => item.group_id == user.group_id,
        crate::models::research::ResearchVisibility::Private => false,
    };

    if allowed {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}
