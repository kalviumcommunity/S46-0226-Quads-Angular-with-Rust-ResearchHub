use crate::models::research::{ResearchItemType, ResearchVisibility};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResearchVersion {
    pub id: Uuid,
    pub research_item_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub description: String,
    pub r#type: ResearchItemType,
    pub visibility: ResearchVisibility,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub file_checksum: Option<String>,
    pub changed_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
