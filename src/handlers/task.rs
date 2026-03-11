use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use log::info;

use crate::AppState;
use crate::errors::AppError;
use crate::models::task::{CreateTaskRequest, SearchTasksQuery, Task, TaskSummary};

#[utoipa::path(
    post,
    path = "/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created"),
        (status = 400, description = "Invalid request")
    ),
    tag = "Tasks"
)]
#[post("/tasks")]
pub async fn create_task(
    state: web::Data<AppState>,
    body: web::Json<CreateTaskRequest>,
) -> Result<HttpResponse, AppError> {
    let req = body.into_inner();

    if req.title.trim().is_empty()
        || req.incident_id.trim().is_empty()
        || req.required_skills.is_empty()
        || req.location_id.trim().is_empty()
        || req.volunteers_needed <= 0
    {
        return Err(AppError::BadRequest(
            "missing or invalid required fields".to_string(),
        ));
    }

    let valid_urgency = ["low", "medium", "high", "critical"];
    if !valid_urgency.contains(&req.urgency.as_str()) {
        return Err(AppError::BadRequest(
            "urgency must be one of: low, medium, high, critical".to_string(),
        ));
    }

    let task = Task {
        task_id: format!("TASK-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()),
        incident_id: req.incident_id,
        title: req.title,
        required_skills: req.required_skills,
        location_id: req.location_id,
        volunteers_needed: req.volunteers_needed,
        volunteers_matched: 0,
        urgency: req.urgency,
        status: "open".to_string(),
        created_at: Utc::now(),
    };

    state.db.create_task(&task).await?;

    info!("[TASK] created: {}", task.task_id);

    Ok(HttpResponse::Created().json(serde_json::json!({
        "task_id": task.task_id,
        "status": task.status,
        "created_at": task.created_at,
    })))
}

#[utoipa::path(
    get,
    path = "/tasks",
    params(
        ("location_id" = Option<String>, Query, description = "Filter by location"),
        ("required_skills" = Option<String>, Query, description = "Filter by skill"),
        ("status" = Option<String>, Query, description = "Filter by status (default: open)")
    ),
    responses(
        (status = 200, description = "List of tasks", body = Vec<TaskSummary>),
        (status = 400, description = "Invalid status value")
    ),
    tag = "Tasks"
)]
#[get("/tasks")]
pub async fn search_tasks(
    state: web::Data<AppState>,
    query: web::Query<SearchTasksQuery>,
) -> Result<HttpResponse, AppError> {
    let status = query.status.as_deref().unwrap_or("open");

    let valid_statuses = ["open", "partially_filled", "filled", "completed", "cancelled"];
    if !valid_statuses.contains(&status) {
        return Err(AppError::BadRequest("invalid status value".to_string()));
    }

    let tasks = state
        .db
        .list_tasks(
            query.location_id.as_deref(),
            query.required_skills.as_deref(),
            status,
        )
        .await?;

    let summaries: Vec<TaskSummary> = tasks
        .into_iter()
        .map(|t| TaskSummary {
            task_id: t.task_id,
            title: t.title,
            urgency: t.urgency,
            volunteers_needed: t.volunteers_needed,
            status: t.status,
        })
        .collect();

    Ok(HttpResponse::Ok().json(summaries))
}
