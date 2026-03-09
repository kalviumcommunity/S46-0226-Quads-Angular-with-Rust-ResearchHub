use crate::{
    app_state::AppState,
    error::AppError,
    middleware::auth::AuthenticatedUser,
    models::{research::ResearchItem, user::Role},
};
use axum::{
    extract::{Extension, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub r#type: Option<String>,
    pub visibility: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub page: u32,
    pub limit: u32,
    pub results: Vec<ResearchItem>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(search))
}

pub async fn search(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        "SELECT id, title, description, type, owner_id, version, visibility, file_url, file_name, file_size_bytes, mime_type, file_checksum, doi, citation_authors, citation_year, institution_id, group_id, created_at, updated_at
         FROM research_items",
    );

    let mut has_where = false;

    if let Some(term) = query.q {
        let like_term = format!("%{}%", term);
        builder.push(if has_where { " AND (" } else { " WHERE (" });
        builder.push("title ILIKE ");
        builder.push_bind(like_term.clone());
        builder.push(" OR description ILIKE ");
        builder.push_bind(like_term);
        builder.push(" OR COALESCE(doi, '') ILIKE ");
        builder.push_bind(format!("%{}%", term));
        builder.push(")");
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

    if user.role != Role::Admin {
        builder.push(if has_where { " AND (" } else { " WHERE (" });
        builder.push("owner_id = ");
        builder.push_bind(user.id);
        builder.push(" OR visibility = 'public'::research_visibility");

        if let Some(institution_id) = user.institution_id {
            builder.push(" OR (visibility = 'institution'::research_visibility AND institution_id = ");
            builder.push_bind(institution_id);
            builder.push(")");
        }

        if let Some(group_id) = user.group_id {
            builder.push(" OR (visibility = 'group'::research_visibility AND group_id = ");
            builder.push_bind(group_id);
            builder.push(")");
        }

        builder.push(")");
    }

    builder.push(" ORDER BY updated_at DESC LIMIT ");
    builder.push_bind(limit as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset as i64);

    let results = builder
        .build_query_as::<ResearchItem>()
        .fetch_all(&state.db_pool)
        .await?;

    Ok(Json(SearchResponse {
        page,
        limit,
        results,
    }))
}
