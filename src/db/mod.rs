use sqlx::PgPool;
use chrono::{DateTime, Utc};

use crate::models::volunteer::Volunteer;
use crate::models::task::Task;
use crate::models::match_model::Match;
use crate::errors::AppError;

pub struct Db {
    pub pool: PgPool,
}

impl Db {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Volunteer ────────────────────────────────────────────────────────────

    pub async fn create_volunteer(&self, v: &Volunteer) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO volunteers \
             (volunteer_id, name, phone, skills, area, availability, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&v.volunteer_id)
        .bind(&v.name)
        .bind(&v.phone)
        .bind(&v.skills)
        .bind(&v.area)
        .bind(&v.availability)
        .bind(v.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn get_volunteer(&self, volunteer_id: &str) -> Result<Option<Volunteer>, AppError> {
        sqlx::query_as::<_, Volunteer>(
            "SELECT volunteer_id, name, phone, skills, area, availability, \
             last_lat, last_lng, location_updated_at, created_at \
             FROM volunteers WHERE volunteer_id = $1",
        )
        .bind(volunteer_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn update_volunteer_location(
        &self,
        volunteer_id: &str,
        lat: f64,
        lng: f64,
        updated_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let rows = sqlx::query(
            "UPDATE volunteers \
             SET last_lat = $1, last_lng = $2, location_updated_at = $3 \
             WHERE volunteer_id = $4",
        )
        .bind(lat)
        .bind(lng)
        .bind(updated_at)
        .bind(volunteer_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .rows_affected();

        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "volunteer {} not found",
                volunteer_id
            )));
        }
        Ok(())
    }

    pub async fn update_volunteer_availability(
        &self,
        volunteer_id: &str,
        availability: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE volunteers SET availability = $1 WHERE volunteer_id = $2")
            .bind(availability)
            .bind(volunteer_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    // ── Task ─────────────────────────────────────────────────────────────────

    pub async fn create_task(&self, t: &Task) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO tasks \
             (task_id, incident_id, title, required_skills, location_id, \
              volunteers_needed, volunteers_matched, urgency, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 0, $7, $8, $9)",
        )
        .bind(&t.task_id)
        .bind(&t.incident_id)
        .bind(&t.title)
        .bind(&t.required_skills)
        .bind(&t.location_id)
        .bind(t.volunteers_needed)
        .bind(&t.urgency)
        .bind(&t.status)
        .bind(t.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Option<Task>, AppError> {
        sqlx::query_as::<_, Task>(
            "SELECT task_id, incident_id, title, required_skills, location_id, \
             volunteers_needed, volunteers_matched, urgency, status, created_at \
             FROM tasks WHERE task_id = $1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn list_tasks(
        &self,
        location_id: Option<&str>,
        required_skill: Option<&str>,
        status: &str,
    ) -> Result<Vec<Task>, AppError> {
        // Build WHERE clauses dynamically
        let mut conditions = vec!["status = $1"];
        let mut param_idx = 2usize;
        let mut loc_idx = 0usize;
        let mut skill_idx = 0usize;

        if location_id.is_some() {
            conditions.push("location_id = $2"); // will be adjusted below
            loc_idx = param_idx;
            param_idx += 1;
        }
        if required_skill.is_some() {
            skill_idx = param_idx;
            param_idx += 1;
        }

        // Rebuild with correct $N placeholders
        let mut conditions2: Vec<String> = vec!["status = $1".to_string()];
        let mut next = 2usize;
        let mut actual_loc_idx = 0usize;
        let mut actual_skill_idx = 0usize;
        if location_id.is_some() {
            conditions2.push(format!("location_id = ${}", next));
            actual_loc_idx = next;
            next += 1;
        }
        if required_skill.is_some() {
            conditions2.push(format!("${} = ANY(required_skills)", next));
            actual_skill_idx = next;
        }

        let sql = format!(
            "SELECT task_id, incident_id, title, required_skills, location_id, \
             volunteers_needed, volunteers_matched, urgency, status, created_at \
             FROM tasks WHERE {}",
            conditions2.join(" AND ")
        );

        let mut q = sqlx::query_as::<_, Task>(&sql).bind(status);
        if let Some(loc) = location_id {
            q = q.bind(loc);
        }
        if let Some(skill) = required_skill {
            q = q.bind(skill);
        }
        // suppress unused warnings
        let _ = (loc_idx, skill_idx, param_idx, actual_loc_idx, actual_skill_idx);

        q.fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Atomically increment volunteers_matched and update status using a transaction.
    pub async fn increment_volunteers_matched(
        &self,
        task_id: &str,
        volunteers_needed: i64,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Lock row
        let row: (i64,) = sqlx::query_as(
            "SELECT volunteers_matched FROM tasks WHERE task_id = $1 FOR UPDATE",
        )
        .bind(task_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("task {} not found", task_id)))?;

        let new_matched = row.0 + 1;
        if new_matched > volunteers_needed {
            tx.rollback().await.ok();
            return Err(AppError::UnprocessableEntity(
                "task is already filled".to_string(),
            ));
        }

        let new_status = if new_matched >= volunteers_needed {
            "filled"
        } else {
            "partially_filled"
        };

        sqlx::query(
            "UPDATE tasks SET volunteers_matched = $1, status = $2 WHERE task_id = $3",
        )
        .bind(new_matched)
        .bind(new_status)
        .bind(task_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        tx.commit().await.map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    // ── Match ────────────────────────────────────────────────────────────────

    pub async fn create_match(&self, m: &Match) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO matches (match_id, task_id, volunteer_id, status, matched_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&m.match_id)
        .bind(&m.task_id)
        .bind(&m.volunteer_id)
        .bind(&m.status)
        .bind(m.matched_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn find_match_by_task_volunteer(
        &self,
        task_id: &str,
        volunteer_id: &str,
    ) -> Result<Option<Match>, AppError> {
        sqlx::query_as::<_, Match>(
            "SELECT match_id, task_id, volunteer_id, status, matched_at \
             FROM matches WHERE task_id = $1 AND volunteer_id = $2 LIMIT 1",
        )
        .bind(task_id)
        .bind(volunteer_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }
}
