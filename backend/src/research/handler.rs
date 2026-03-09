use crate::{
    app_state::AppState,
    error::AppError,
    models::research::{CreateResearchItemInput, ResearchItem, UpdateResearchItemInput},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_item).get(list_items))
        .route("/:id", get(get_item).put(update_item).delete(delete_item))
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateResearchItemInput>,
) -> Result<impl IntoResponse, AppError> {
    let item = sqlx::query_as::<_, ResearchItem>(
        "INSERT INTO research_items (title, description, type, owner_id, visibility, file_url, institution_id, group_id)
         VALUES ($1, $2, $3::research_item_type, $4, $5::research_visibility, $6, $7, $8)
         RETURNING id, title, description, type, owner_id, version, visibility, file_url, institution_id, group_id, created_at, updated_at",
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(format_item_type(&payload.r#type))
    .bind(payload.owner_id)
    .bind(format_visibility(&payload.visibility))
    .bind(payload.file_url)
    .bind(payload.institution_id)
    .bind(payload.group_id)
    .fetch_one(&state.db_pool)
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn list_items(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let items = sqlx::query_as::<_, ResearchItem>(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, institution_id, group_id, created_at, updated_at
         FROM research_items
         ORDER BY created_at DESC",
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(Json(items))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let item = sqlx::query_as::<_, ResearchItem>(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, institution_id, group_id, created_at, updated_at
         FROM research_items
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateResearchItemInput>,
) -> Result<impl IntoResponse, AppError> {
    let current = sqlx::query_as::<_, ResearchItem>(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, institution_id, group_id, created_at, updated_at
         FROM research_items
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let updated = sqlx::query_as::<_, ResearchItem>(
        "UPDATE research_items
         SET title = $2,
             description = $3,
             type = $4::research_item_type,
             visibility = $5::research_visibility,
             file_url = $6,
             institution_id = $7,
             group_id = $8,
             version = version + 1,
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, title, description, type, owner_id, version, visibility, file_url, institution_id, group_id, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.title.unwrap_or(current.title))
    .bind(payload.description.unwrap_or(current.description))
    .bind(format_item_type(&payload.r#type.unwrap_or(current.r#type)))
    .bind(format_visibility(&payload.visibility.unwrap_or(current.visibility)))
    .bind(payload.file_url.or(current.file_url))
    .bind(payload.institution_id.or(current.institution_id))
    .bind(payload.group_id.or(current.group_id))
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(updated))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM research_items WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

fn format_item_type(value: &crate::models::research::ResearchItemType) -> String {
    match value {
        crate::models::research::ResearchItemType::Paper => "paper".to_string(),
        crate::models::research::ResearchItemType::Dataset => "dataset".to_string(),
        crate::models::research::ResearchItemType::Code => "code".to_string(),
    }
}

fn format_visibility(value: &crate::models::research::ResearchVisibility) -> String {
    match value {
        crate::models::research::ResearchVisibility::Private => "private".to_string(),
        crate::models::research::ResearchVisibility::Group => "group".to_string(),
        crate::models::research::ResearchVisibility::Institution => "institution".to_string(),
        crate::models::research::ResearchVisibility::Public => "public".to_string(),
    }
}
