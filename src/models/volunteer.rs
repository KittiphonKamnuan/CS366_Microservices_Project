use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct Volunteer {
    pub volunteer_id: String,
    pub name: String,
    pub phone: String,
    pub skills: Vec<String>,
    pub area: String,
    pub availability: String,
    pub last_lat: Option<f64>,
    pub last_lng: Option<f64>,
    pub location_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterVolunteerRequest {
    /// ชื่อ-นามสกุล
    pub name: String,
    /// เบอร์โทร
    pub phone: String,
    /// ทักษะ เช่น ["driving", "boat"]
    pub skills: Vec<String>,
    /// พื้นที่ที่สะดวก (location_id)
    pub area: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterVolunteerResponse {
    pub volunteer_id: String,
    pub availability: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLocationRequest {
    /// ละติจูด (5.5 – 20.5)
    pub lat: f64,
    /// ลองจิจูด (97.5 – 105.7)
    pub lng: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationResponse {
    pub volunteer_id: String,
    pub lat: f64,
    pub lng: f64,
    pub location_updated_at: DateTime<Utc>,
}
