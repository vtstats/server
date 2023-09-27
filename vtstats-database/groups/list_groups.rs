use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::{PgPool, Result};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct Group {
    pub group_id: String,
    #[serde(default)]
    pub root: bool,
    pub native_name: String,
    #[serde(default)]
    pub english_name: Option<String>,
    #[serde(default)]
    pub japanese_name: Option<String>,
    pub children: Vec<String>,
}

pub async fn list_groups(pool: &PgPool) -> Result<Vec<Group>> {
    let query = sqlx::query_as!(
        Group,
        "SELECT group_id, root, native_name, english_name, japanese_name, children as \"children!: _ \" FROM groups",
    )
    .fetch_all(pool);

    let res = crate::otel::execute_query!("SELECT", "groups", query)?;

    Ok(res)
}
