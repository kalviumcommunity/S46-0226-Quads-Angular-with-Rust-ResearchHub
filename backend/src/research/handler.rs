use crate::{
    app_state::AppState,
    error::AppError,
    middleware::auth::AuthenticatedUser,
    models::{
        research::{
            CreateResearchItemInput, ResearchItem, ResearchVisibility, UpdateResearchItemInput,
        },
        user::Role,
    },
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListResearchQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub owner_id: Option<Uuid>,
    pub r#type: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListResearchResponse {
    pub page: u32,
    pub limit: u32,
    pub items: Vec<ResearchItem>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_item).get(list_items))
        .route("/:id", get(get_item).put(update_item).delete(delete_item))
}

pub async fn create_item(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateResearchItemInput>,
) -> Result<impl IntoResponse, AppError> {
    if current_user.role != Role::Admin && payload.owner_id != current_user.id {
        return Err(AppError::Forbidden);
    }

    if current_user.role == Role::Student && payload.visibility == ResearchVisibility::Public {
        return Err(AppError::Forbidden);
    }

    let item = sqlx::query_as::<_, ResearchItem>(
        "INSERT INTO research_items (title, description, type, owner_id, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id)
         VALUES ($1, $2, $3::research_item_type, $4, $5::research_visibility, $6, $7, $8, $9, $10, $11, $12)
         RETURNING id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id, created_at, updated_at",
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(format_item_type(&payload.r#type))
    .bind(payload.owner_id)
    .bind(format_visibility(&payload.visibility))
    .bind(payload.file_url)
    .bind(payload.file_name)
    .bind(payload.file_size_bytes)
    .bind(payload.mime_type)
    .bind(payload.file_checksum)
    .bind(payload.institution_id)
    .bind(payload.group_id)
    .fetch_one(&state.db_pool)
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn list_items(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthenticatedUser>,
    Query(query): Query<ListResearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id, created_at, updated_at
         FROM research_items",
    );

    let mut has_where = false;
    if let Some(owner_id) = query.owner_id {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("owner_id = ");
        builder.push_bind(owner_id);
        has_where = true;
    }

    if let Some(item_type) = query.r#type {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("type = ");
        builder.push_bind(item_type);
        builder.push("::research_item_type");
        has_where = true;
    }

    if let Some(visibility) = query.visibility {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("visibility = ");
        builder.push_bind(visibility);
        builder.push("::research_visibility");
        has_where = true;
    }

    if current_user.role != Role::Admin {
        builder.push(if has_where { " AND (" } else { " WHERE (" });
        builder.push("owner_id = ");
        builder.push_bind(current_user.id);
        builder.push(" OR visibility = 'public'::research_visibility");

        if let Some(institution_id) = current_user.institution_id {
            builder.push(" OR (visibility = 'institution'::research_visibility AND institution_id = ");
            builder.push_bind(institution_id);
            builder.push(")");
        }

        if let Some(group_id) = current_user.group_id {
            builder.push(" OR (visibility = 'group'::research_visibility AND group_id = ");
            builder.push_bind(group_id);
            builder.push(")");
        }

        builder.push(")");
    }

    builder.push(" ORDER BY created_at DESC LIMIT ");
    builder.push_bind(limit as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset as i64);

    let items = builder
        .build_query_as::<ResearchItem>()
        .fetch_all(&state.db_pool)
        .await?;

    Ok(Json(ListResearchResponse { page, limit, items }))
}

pub async fn get_item(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let item = sqlx::query_as::<_, ResearchItem>(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id, created_at, updated_at
         FROM research_items
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if !can_view_item(&current_user, &item) {
        return Err(AppError::Forbidden);
    }

    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateResearchItemInput>,
) -> Result<impl IntoResponse, AppError> {
    let current = sqlx::query_as::<_, ResearchItem>(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id, created_at, updated_at
         FROM research_items
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if !can_manage_item(&current_user, &current) {
        return Err(AppError::Forbidden);
    }

    let updated = sqlx::query_as::<_, ResearchItem>(
        "UPDATE research_items
         SET title = $2,
             description = $3,
             type = $4::research_item_type,
             visibility = $5::research_visibility,
             file_url = $6,
             file_name = $7,
             file_size_bytes = $8,
             mime_type = $9,
             file_checksum = $10,
             institution_id = $11,
             group_id = $12,
             version = version + 1,
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.title.unwrap_or(current.title))
    .bind(payload.description.unwrap_or(current.description))
    .bind(format_item_type(&payload.r#type.unwrap_or(current.r#type)))
    .bind(format_visibility(&payload.visibility.unwrap_or(current.visibility)))
    .bind(payload.file_url.or(current.file_url))
    .bind(payload.file_name.or(current.file_name))
    .bind(payload.file_size_bytes.or(current.file_size_bytes))
    .bind(payload.mime_type.or(current.mime_type))
    .bind(payload.file_checksum.or(current.file_checksum))
    .bind(payload.institution_id.or(current.institution_id))
    .bind(payload.group_id.or(current.group_id))
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(updated))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let current = sqlx::query_as::<_, ResearchItem>(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, institution_id, group_id, created_at, updated_at
         FROM research_items
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if !can_manage_item(&current_user, &current) {
        return Err(AppError::Forbidden);
    }

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

fn can_view_item(user: &AuthenticatedUser, item: &ResearchItem) -> bool {
    if user.role == Role::Admin || item.owner_id == user.id {
        return true;
    }

    match item.visibility {
        ResearchVisibility::Public => true,
        ResearchVisibility::Institution => item.institution_id == user.institution_id,
        ResearchVisibility::Group => item.group_id == user.group_id,
        ResearchVisibility::Private => false,
    }
}

fn can_manage_item(user: &AuthenticatedUser, item: &ResearchItem) -> bool {
    if user.role == Role::Admin || item.owner_id == user.id {
        return true;
    }

    user.role == Role::Faculty
        && item.visibility != ResearchVisibility::Private
        && item.institution_id.is_some()
        && item.institution_id == user.institution_id
}
