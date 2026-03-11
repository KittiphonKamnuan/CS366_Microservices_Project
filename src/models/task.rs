use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Task {
    pub task_id: String,
    pub incident_id: String,
    pub title: String,
    pub required_skills: Vec<String>,
    pub location_id: String,
    pub volunteers_needed: i64,
    pub volunteers_matched: i64,
    pub urgency: String, // "low" | "medium" | "high" | "critical"
    pub status: String,  // "open" | "partially_filled" | "filled" | "completed" | "cancelled"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub incident_id: String,
    pub title: String,
    pub required_skills: Vec<String>,
    pub location_id: String,
    pub volunteers_needed: i64,
    pub urgency: String,
}

#[derive(Debug, Serialize)]
pub struct TaskSummary {
    pub task_id: String,
    pub title: String,
    pub urgency: String,
    pub volunteers_needed: i64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchTasksQuery {
    pub location_id: Option<String>,
    pub required_skills: Option<String>,
    pub status: Option<String>,
}
