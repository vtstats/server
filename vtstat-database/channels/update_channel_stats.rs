use sqlx::{PgPool, Postgres, QueryBuilder, Result};

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
}

pub async fn update_channel_stats(
    iter: impl Iterator<Item = ChannelStatsSummary>,
    pool: &PgPool,
) -> Result<()> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "UPDATE channels AS c SET \
        view = n.view, \
        view_1d_ago = n.view_1d_ago, \
        view_7d_ago = n.view_7d_ago, \
        view_30d_ago = n.view_30d_ago, \
        subscriber = n.subscriber, \
        subscriber_1d_ago = n.subscriber_1d_ago, \
        subscriber_7d_ago = n.subscriber_7d_ago, \
        subscriber_30d_ago = n.subscriber_30d_ago FROM (",
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
            .push_bind(row.subscriber_30d_ago.unwrap_or_default());
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
        subscriber_30d_ago) \
        WHERE c.channel_id = n.channel_id",
    );

    let query = query_builder.build().execute(pool);

    crate::otel::instrument("UPDATE", "channels", query).await?;

    Ok(())
}
