use sqlx::{PgPool, Postgres, QueryBuilder, Result};

use super::Group;

pub async fn update_groups(groups: Vec<Group>, pool: PgPool) -> Result<()> {
    let mut tx = pool.begin().await?;

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO groups (group_id, native_name, english_name, japanese_name, children, root) ",
    );

    query_builder.push_values(
        groups.iter().filter(|group| !group.children.is_empty()),
        |mut b, group| {
            b.push_bind(&group.group_id)
                .push_bind(&group.native_name)
                .push_bind(&group.english_name)
                .push_bind(&group.japanese_name)
                .push_bind(&group.children)
                .push_bind(group.root);
        },
    );

    query_builder.push(
        "ON CONFLICT (group_id) DO UPDATE \
        SET group_id = excluded.group_id, \
        native_name = excluded.native_name, \
        english_name = excluded.english_name, \
        japanese_name = excluded.japanese_name, \
        children = excluded.children, \
        root = excluded.root",
    );

    let query = query_builder.build().execute(&mut *tx);

    crate::otel::instrument("INSERT", "groups", query).await?;

    let delete: Vec<_> = groups
        .into_iter()
        .filter_map(|g| g.children.is_empty().then(|| g.group_id))
        .collect();

    if !delete.is_empty() {
        let query =
            sqlx::query!("DELETE FROM groups WHERE group_id = ANY($1)", &delete).execute(&mut *tx);

        crate::otel::instrument("DELETE", "groups", query).await?;
    }

    tx.commit().await
}
