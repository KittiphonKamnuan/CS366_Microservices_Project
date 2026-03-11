use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Match {
    pub match_id: String,
    pub task_id: String,
    pub volunteer_id: String,
    pub status: String, // "pending" | "accepted" | "declined" | "completed"
    pub matched_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MatchVolunteerRequest {
    pub volunteer_id: String,
}

#[derive(Debug, Serialize)]
pub struct MatchResponse {
    pub match_id: String,
    pub status: String,
    pub matched_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
