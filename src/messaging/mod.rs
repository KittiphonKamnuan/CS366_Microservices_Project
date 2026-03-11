use aws_sdk_sns::Client as SnsClient;
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;
use log::{info, error};

use crate::errors::AppError;

pub struct Messenger {
    pub sns: SnsClient,
    pub location_topic_arn: String,
    pub match_topic_arn: String,
}

impl Messenger {
    pub fn new(sns: SnsClient) -> Self {
        Self {
            sns,
            location_topic_arn: std::env::var("SNS_LOCATION_TOPIC_ARN")
                .unwrap_or_default(),
            match_topic_arn: std::env::var("SNS_MATCH_TOPIC_ARN")
                .unwrap_or_default(),
        }
    }

    /// Publish volunteer.location_updated event to SNS
    pub async fn publish_location_updated(
        &self,
        volunteer_id: &str,
        lat: f64,
        lng: f64,
    ) -> Result<(), AppError> {
        let message_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let body: Value = json!({
            "event": "volunteer.location_updated",
            "volunteer_id": volunteer_id,
            "lat": lat,
            "lng": lng,
            "timestamp": now,
        });

        let message_attrs = build_sns_attributes(
            "volunteer.location_updated",
            &message_id,
            &now,
            None,
        );

        if self.location_topic_arn.is_empty() {
            info!(
                "[MOCK SNS] volunteer.location_updated | volunteer_id={} lat={} lng={}",
                volunteer_id, lat, lng
            );
            return Ok(());
        }

        self.sns
            .publish()
            .topic_arn(&self.location_topic_arn)
            .message(serde_json::to_string(&body).unwrap())
            .set_message_attributes(Some(message_attrs))
            .send()
            .await
            .map_err(|e| {
                error!("SNS publish failed: {}", e);
                AppError::Internal(format!("SNS publish failed: {}", e))
            })?;

        info!(
            "[SNS] published volunteer.location_updated | messageId={}",
            message_id
        );
        Ok(())
    }

    /// Publish match.status_changed event to SNS
    pub async fn publish_match_status_changed(
        &self,
        match_id: &str,
        task_id: &str,
        volunteer_id: &str,
        status: &str,
    ) -> Result<(), AppError> {
        let message_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let body: Value = json!({
            "event": "match.status_changed",
            "match_id": match_id,
            "task_id": task_id,
            "volunteer_id": volunteer_id,
            "status": status,
            "timestamp": now,
        });

        let message_attrs = build_sns_attributes(
            "match.status_changed",
            &message_id,
            &now,
            None,
        );

        if self.match_topic_arn.is_empty() {
            info!(
                "[MOCK SNS] match.status_changed | match_id={} status={}",
                match_id, status
            );
            return Ok(());
        }

        self.sns
            .publish()
            .topic_arn(&self.match_topic_arn)
            .message(serde_json::to_string(&body).unwrap())
            .set_message_attributes(Some(message_attrs))
            .send()
            .await
            .map_err(|e| {
                error!("SNS publish failed: {}", e);
                AppError::Internal(format!("SNS publish failed: {}", e))
            })?;

        info!(
            "[SNS] published match.status_changed | messageId={}",
            message_id
        );
        Ok(())
    }
}

fn build_sns_attributes(
    message_type: &str,
    message_id: &str,
    sent_at: &str,
    trace_id: Option<&str>,
) -> std::collections::HashMap<String, aws_sdk_sns::types::MessageAttributeValue> {
    use aws_sdk_sns::types::MessageAttributeValue;
    let mut attrs = std::collections::HashMap::new();

    let string_type = "String".to_string();

    attrs.insert(
        "messageType".to_string(),
        MessageAttributeValue::builder()
            .data_type(string_type.clone())
            .string_value(message_type.to_string())
            .build()
            .unwrap(),
    );
    attrs.insert(
        "messageId".to_string(),
        MessageAttributeValue::builder()
            .data_type(string_type.clone())
            .string_value(message_id.to_string())
            .build()
            .unwrap(),
    );
    attrs.insert(
        "sentAt".to_string(),
        MessageAttributeValue::builder()
            .data_type(string_type.clone())
            .string_value(sent_at.to_string())
            .build()
            .unwrap(),
    );
    if let Some(tid) = trace_id {
        attrs.insert(
            "traceId".to_string(),
            MessageAttributeValue::builder()
                .data_type(string_type)
                .string_value(tid.to_string())
                .build()
                .unwrap(),
        );
    }
    attrs
}
