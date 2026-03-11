use actix_web::{get, patch, post, web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use log::info;

use crate::AppState;
use crate::errors::AppError;
use crate::models::volunteer::{
    RegisterVolunteerRequest, RegisterVolunteerResponse,
    UpdateLocationRequest, LocationResponse, Volunteer,
};

#[utoipa::path(
    post,
    path = "/volunteers",
    request_body = RegisterVolunteerRequest,
    responses(
        (status = 201, description = "Volunteer registered", body = RegisterVolunteerResponse),
        (status = 400, description = "Missing required fields")
    ),
    tag = "Volunteers"
)]
#[post("/volunteers")]
pub async fn register_volunteer(
    state: web::Data<AppState>,
    body: web::Json<RegisterVolunteerRequest>,
) -> Result<HttpResponse, AppError> {
    let req = body.into_inner();

    if req.name.trim().is_empty()
        || req.phone.trim().is_empty()
        || req.skills.is_empty()
        || req.area.trim().is_empty()
    {
        return Err(AppError::BadRequest(
            "missing required fields: name, phone, skills, area".to_string(),
        ));
    }

    let now = Utc::now();
    let volunteer = Volunteer {
        volunteer_id: format!("VOL-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()),
        name: req.name,
        phone: req.phone,
        skills: req.skills,
        area: req.area,
        availability: "available".to_string(),
        last_lat: None,
        last_lng: None,
        location_updated_at: None,
        created_at: now,
    };

    state.db.create_volunteer(&volunteer).await?;

    info!("[VOLUNTEER] registered: {}", volunteer.volunteer_id);

    Ok(HttpResponse::Created().json(RegisterVolunteerResponse {
        volunteer_id: volunteer.volunteer_id,
        availability: volunteer.availability,
        created_at: now,
    }))
}

#[utoipa::path(
    patch,
    path = "/volunteers/{volunteer_id}/location",
    params(
        ("volunteer_id" = String, Path, description = "Volunteer ID")
    ),
    request_body = UpdateLocationRequest,
    responses(
        (status = 200, description = "Location updated", body = LocationResponse),
        (status = 400, description = "Invalid coordinates"),
        (status = 404, description = "Volunteer not found")
    ),
    tag = "Volunteers"
)]
#[patch("/volunteers/{volunteer_id}/location")]
pub async fn update_location(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateLocationRequest>,
) -> Result<HttpResponse, AppError> {
    let volunteer_id = path.into_inner();
    let req = body.into_inner();

    if !(5.5..=20.5).contains(&req.lat) || !(97.5..=105.7).contains(&req.lng) {
        return Err(AppError::BadRequest(
            "invalid coordinates: lat/lng out of Thailand bounds".to_string(),
        ));
    }

    let now = Utc::now();

    state
        .db
        .update_volunteer_location(&volunteer_id, req.lat, req.lng, now)
        .await?;

    let messenger = state.messenger.clone();
    let vid = volunteer_id.clone();
    tokio::spawn(async move {
        if let Err(e) = messenger
            .publish_location_updated(&vid, req.lat, req.lng)
            .await
        {
            log::error!("async publish failed: {}", e);
        }
    });

    info!(
        "[VOLUNTEER] location updated: {} lat={} lng={}",
        volunteer_id, req.lat, req.lng
    );

    Ok(HttpResponse::Ok().json(LocationResponse {
        volunteer_id,
        lat: req.lat,
        lng: req.lng,
        location_updated_at: now,
    }))
}

#[utoipa::path(
    get,
    path = "/volunteers/{volunteer_id}/location",
    params(
        ("volunteer_id" = String, Path, description = "Volunteer ID")
    ),
    responses(
        (status = 200, description = "Current GPS location", body = LocationResponse),
        (status = 404, description = "Volunteer not found or no GPS data")
    ),
    tag = "Volunteers"
)]
#[get("/volunteers/{volunteer_id}/location")]
pub async fn get_location(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let volunteer_id = path.into_inner();

    let volunteer = state
        .db
        .get_volunteer(&volunteer_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("volunteer {} not found", volunteer_id)))?;

    let (lat, lng) = match (volunteer.last_lat, volunteer.last_lng) {
        (Some(lat), Some(lng)) => (lat, lng),
        _ => return Err(AppError::NotFound("no GPS data yet".to_string())),
    };

    Ok(HttpResponse::Ok().json(LocationResponse {
        volunteer_id: volunteer.volunteer_id,
        lat,
        lng,
        location_updated_at: volunteer.location_updated_at.unwrap_or_else(Utc::now),
    }))
}
