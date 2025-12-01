#[cfg(feature = "ssr")]
use axum::Json;

#[cfg(feature = "ssr")]
pub async fn get_usage_stats() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "messages_sent": 0,
        "documents_created": 0,
        "total_users": 1,
        "storage_used": 0,
        "limit_exceeded": false
    }))
}
