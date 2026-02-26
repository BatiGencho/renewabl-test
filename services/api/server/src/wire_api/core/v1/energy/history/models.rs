use serde::Serialize;
use utoipa::ToSchema;

/// A single query history entry
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryHistoryEntry {
    pub id: uuid::Uuid,
    pub aggregation_type: String,
    #[schema(example = "2025-01-01T00:00:00Z")]
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(example = "2025-04-01T00:00:00Z")]
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response containing the last 10 queries
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponse {
    pub queries: Vec<QueryHistoryEntry>,
}
