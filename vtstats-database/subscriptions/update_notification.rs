use serde_json::json;
use sqlx::{postgres::PgQueryResult, PgPool, Result};

pub async fn update_discord_notification(
    notification_id: i32,
    message_id: String,
    start_message_id: Option<String>,
    end_message_id: Option<String>,
    pool: &PgPool,
) -> Result<PgQueryResult> {
    let payload = json!({
        "message_id": message_id,
        "start_message_id": start_message_id,
        "end_message_id": end_message_id,
    });

    let query = sqlx::query!(
        "UPDATE notifications \
        SET payload = payload || $2, updated_at = NOW() \
        WHERE notification_id = $1",
        notification_id,
        payload,
    )
    .execute(pool);

    crate::otel::execute_query!("UPDATE", "notifications", query)
}
