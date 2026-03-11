use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Volunteer {
    pub volunteer_id: String,
    pub name: String,
    pub phone: String,
    pub skills: Vec<String>,
    pub area: String,
    pub availability: String, // "available" | "busy" | "inactive"
    pub last_lat: Option<f64>,
    pub last_lng: Option<f64>,
    pub location_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterVolunteerRequest {
    pub name: String,
    pub phone: String,
    pub skills: Vec<String>,
    pub area: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterVolunteerResponse {
    pub volunteer_id: String,
    pub availability: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize)]
pub struct LocationResponse {
    pub volunteer_id: String,
    pub lat: f64,
    pub lng: f64,
    pub location_updated_at: DateTime<Utc>,
}
