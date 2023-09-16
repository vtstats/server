use std::collections::HashMap;

use sqlx::{types::Json, PgPool, Postgres, QueryBuilder, Result};

#[derive(Debug)]
pub struct ChannelStatsSummary {
    pub channel_id: i32,
    pub view: Option<i32>,
    pub view_1d_ago: Option<i32>,
    pub view_7d_ago: Option<i32>,
    pub view_30d_ago: Option<i32>,
    pub subscriber: Option<i32>,
    pub subscriber_1d_ago: Option<i32>,
    pub subscriber_7d_ago: Option<i32>,
    pub subscriber_30d_ago: Option<i32>,
    pub revenue: Option<HashMap<String, f32>>,
    pub revenue_1d_ago: Option<HashMap<String, f32>>,
    pub revenue_7d_ago: Option<HashMap<String, f32>>,
    pub revenue_30d_ago: Option<HashMap<String, f32>>,
}

pub async fn update_channel_stats(
    iter: impl Iterator<Item = ChannelStatsSummary>,
    pool: &PgPool,
) -> Result<()> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "UPDATE channels AS c SET \
        view = COALESCE(n.view, c.view), \
        view_1d_ago = COALESCE(n.view_1d_ago, c.view_1d_ago), \
        view_7d_ago = COALESCE(n.view_7d_ago, c.view_7d_ago), \
        view_30d_ago = COALESCE(n.view_30d_ago, c.view_30d_ago), \
        subscriber = COALESCE(n.subscriber, c.subscriber), \
        subscriber_1d_ago = COALESCE(n.subscriber_1d_ago, c.subscriber_1d_ago), \
        subscriber_7d_ago = COALESCE(n.subscriber_7d_ago, c.subscriber_7d_ago), \
        subscriber_30d_ago = COALESCE(n.subscriber_30d_ago, c.subscriber_30d_ago), \
        revenue = COALESCE(n.revenue, c.revenue), \
        revenue_1d_ago = COALESCE(n.revenue_1d_ago, c.revenue_1d_ago), \
        revenue_7d_ago = COALESCE(n.revenue_7d_ago, c.revenue_7d_ago), \
        revenue_30d_ago = COALESCE(n.revenue_30d_ago, c.revenue_30d_ago) FROM (",
    );

    query_builder.push_values(iter, |mut b, row| {
        b.push_bind(row.channel_id)
            .push_bind(row.view.unwrap_or_default())
            .push_bind(row.view_1d_ago.unwrap_or_default())
            .push_bind(row.view_7d_ago.unwrap_or_default())
            .push_bind(row.view_30d_ago.unwrap_or_default())
            .push_bind(row.subscriber.unwrap_or_default())
            .push_bind(row.subscriber_1d_ago.unwrap_or_default())
            .push_bind(row.subscriber_7d_ago.unwrap_or_default())
            .push_bind(row.subscriber_30d_ago.unwrap_or_default())
            .push_bind(Json(row.revenue.unwrap_or_default()))
            .push_bind(Json(row.revenue_1d_ago.unwrap_or_default()))
            .push_bind(Json(row.revenue_7d_ago.unwrap_or_default()))
            .push_bind(Json(row.revenue_30d_ago.unwrap_or_default()));
    });

    query_builder.push(
        ") AS n( \
        channel_id, \
        view, \
        view_1d_ago, \
        view_7d_ago, \
        view_30d_ago, \
        subscriber, \
        subscriber_1d_ago, \
        subscriber_7d_ago, \
        subscriber_30d_ago, \
        revenue, \
        revenue_1d_ago, \
        revenue_7d_ago, \
        revenue_30d_ago) \
        WHERE c.channel_id = n.channel_id",
    );

    let query = query_builder.build().execute(pool);

    crate::otel::execute_query!("UPDATE", "channels", query)?;

    Ok(())
}
