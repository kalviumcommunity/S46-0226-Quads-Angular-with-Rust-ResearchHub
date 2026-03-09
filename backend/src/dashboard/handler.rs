use crate::{
    app_state::AppState,
    error::AppError,
    middleware::auth::AuthenticatedUser,
    models::user::Role,
};
use axum::{extract::Extension, extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_items: i64,
    pub visible_items: i64,
    pub my_items: i64,
    pub total_comments: i64,
    pub total_users: i64,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/stats", get(stats))
}

pub async fn stats(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, AppError> {
    let total_items =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM research_items")
            .fetch_one(&state.db_pool)
            .await?;

    let my_items =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM research_items WHERE owner_id = $1")
            .bind(user.id)
            .fetch_one(&state.db_pool)
            .await?;

    let visible_items = if user.role == Role::Admin {
        total_items
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::BIGINT
             FROM research_items
             WHERE owner_id = $1
                OR visibility = 'public'::research_visibility
                OR (visibility = 'institution'::research_visibility AND institution_id = $2)
                OR (visibility = 'group'::research_visibility AND group_id = $3)",
        )
        .bind(user.id)
        .bind(user.institution_id)
        .bind(user.group_id)
        .fetch_one(&state.db_pool)
        .await?
    };

    let total_comments =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM comments")
            .fetch_one(&state.db_pool)
            .await?;

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM users")
        .fetch_one(&state.db_pool)
        .await?;

    Ok(Json(DashboardStats {
        total_items,
        visible_items,
        my_items,
        total_comments,
        total_users,
    }))
}
