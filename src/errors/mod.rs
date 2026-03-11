use actix_web::HttpResponse;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(json!({ "error": msg }))
            }
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(json!({ "error": msg }))
            }
            AppError::UnprocessableEntity(msg) => {
                HttpResponse::UnprocessableEntity().json(json!({ "error": msg }))
            }
            AppError::Internal(msg) => {
                HttpResponse::InternalServerError().json(json!({ "error": msg }))
            }
            AppError::Conflict(msg) => {
                HttpResponse::Conflict().json(json!({ "error": msg }))
            }
        }
    }
}
