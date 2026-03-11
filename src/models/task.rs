use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct Task {
    pub task_id: String,
    pub incident_id: String,
    pub title: String,
    pub required_skills: Vec<String>,
    pub location_id: String,
    pub volunteers_needed: i64,
    pub volunteers_matched: i64,
    /// low | medium | high | critical
    pub urgency: String,
    /// open | partially_filled | filled | completed | cancelled
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub incident_id: String,
    pub title: String,
    pub required_skills: Vec<String>,
    pub location_id: String,
    pub volunteers_needed: i64,
    /// low | medium | high | critical
    pub urgency: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskSummary {
    pub task_id: String,
    pub title: String,
    pub urgency: String,
    pub volunteers_needed: i64,
    pub status: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchTasksQuery {
    pub location_id: Option<String>,
    pub required_skills: Option<String>,
    /// default: open
    pub status: Option<String>,
}
