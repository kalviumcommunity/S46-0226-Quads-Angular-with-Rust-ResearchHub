use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "research_item_type", rename_all = "lowercase")]
pub enum ResearchItemType {
    Paper,
    Dataset,
    Code,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "research_visibility", rename_all = "lowercase")]
pub enum ResearchVisibility {
    Private,
    Group,
    Institution,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResearchItem {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub r#type: ResearchItemType,
    pub owner_id: Uuid,
    pub version: i32,
    pub visibility: ResearchVisibility,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub file_checksum: Option<String>,
    pub institution_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateResearchItemInput {
    pub title: String,
    pub description: String,
    pub r#type: ResearchItemType,
    pub owner_id: Uuid,
    pub visibility: ResearchVisibility,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub file_checksum: Option<String>,
    pub institution_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateResearchItemInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<ResearchItemType>,
    pub visibility: Option<ResearchVisibility>,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub file_checksum: Option<String>,
    pub institution_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
}
