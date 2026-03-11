use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct Match {
    pub match_id: String,
    pub task_id: String,
    pub volunteer_id: String,
    /// pending | accepted | declined | completed
    pub status: String,
    pub matched_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MatchVolunteerRequest {
    pub volunteer_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MatchResponse {
    pub match_id: String,
    pub status: String,
    pub matched_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
