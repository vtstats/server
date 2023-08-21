use serde::Serialize;
use sqlx::{PgPool, Result};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    group_id: String,
    root: bool,
    native_name: String,
    english_name: Option<String>,
    japanese_name: Option<String>,
    children: Vec<String>,
}

pub async fn list_groups(pool: &PgPool) -> Result<Vec<Group>> {
    let query = sqlx::query_as!(
        Group,
        "SELECT group_id, root, native_name, english_name, japanese_name, children as \"children!: _ \" FROM groups",
    )
    .fetch_all(pool);

    let res = crate::otel::instrument("SELECT", "groups", query).await?;

    Ok(res)
}
