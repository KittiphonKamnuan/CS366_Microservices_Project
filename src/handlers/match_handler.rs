use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use log::info;

use crate::AppState;
use crate::errors::AppError;
use crate::models::match_model::{Match, MatchResponse, MatchVolunteerRequest};

/// POST /tasks/{task_id}/match — Match volunteer to task (Sync)
#[post("/tasks/{task_id}/match")]
pub async fn match_volunteer(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<MatchVolunteerRequest>,
) -> Result<HttpResponse, AppError> {
    let task_id = path.into_inner();
    let req = body.into_inner();

    if req.volunteer_id.trim().is_empty() {
        return Err(AppError::BadRequest("volunteer_id is required".to_string()));
    }

    // Load task
    let task = state
        .db
        .get_task(&task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("task {} not found", task_id)))?;

    // Task must be open or partially_filled
    if task.status != "open" && task.status != "partially_filled" {
        return Err(AppError::UnprocessableEntity(
            "task is not accepting volunteers".to_string(),
        ));
    }

    // Idempotency check — return existing match if already matched (before any other checks)
    if let Some(existing) = state
        .db
        .find_match_by_task_volunteer(&task_id, &req.volunteer_id)
        .await?
    {
        info!(
            "[MATCH] duplicate match request, returning existing: {}",
            existing.match_id
        );
        return Ok(HttpResponse::Ok().json(MatchResponse {
            match_id: existing.match_id,
            status: existing.status,
            matched_at: existing.matched_at,
            note: Some("match already exists".to_string()),
        }));
    }

    // Load volunteer
    let volunteer = state
        .db
        .get_volunteer(&req.volunteer_id)
        .await?
        .ok_or_else(|| AppError::UnprocessableEntity("volunteer not found".to_string()))?;

    // Check availability
    if volunteer.availability != "available" {
        return Err(AppError::UnprocessableEntity(
            "volunteer is not available or skills do not match".to_string(),
        ));
    }

    // Check skills match
    let has_skill = task
        .required_skills
        .iter()
        .all(|s| volunteer.skills.contains(s));
    if !has_skill {
        return Err(AppError::UnprocessableEntity(
            "volunteer is not available or skills do not match".to_string(),
        ));
    }

    // Check area match
    if volunteer.area != task.location_id {
        return Err(AppError::UnprocessableEntity(
            "volunteer area does not match task location".to_string(),
        ));
    }

    // Increment volunteers_matched with optimistic locking
    state
        .db
        .increment_volunteers_matched(&task_id, task.volunteers_needed)
        .await?;

    // Create match record
    let now = Utc::now();
    let new_match = Match {
        match_id: format!("MATCH-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()),
        task_id: task_id.clone(),
        volunteer_id: req.volunteer_id.clone(),
        status: "pending".to_string(),
        matched_at: now,
    };

    state.db.create_match(&new_match).await?;

    // Mark volunteer as busy
    state
        .db
        .update_volunteer_availability(&req.volunteer_id, "busy")
        .await?;

    // Publish async match.status_changed event
    let messenger = state.messenger.clone();
    let mid = new_match.match_id.clone();
    let tid = task_id.clone();
    let vid = req.volunteer_id.clone();
    tokio::spawn(async move {
        if let Err(e) = messenger
            .publish_match_status_changed(&mid, &tid, &vid, "pending")
            .await
        {
            log::error!("async publish match.status_changed failed: {}", e);
        }
    });

    info!(
        "[MATCH] created: {} (volunteer={} task={})",
        new_match.match_id, req.volunteer_id, task_id
    );

    Ok(HttpResponse::Created().json(MatchResponse {
        match_id: new_match.match_id,
        status: new_match.status,
        matched_at: now,
        note: None,
    }))
}
