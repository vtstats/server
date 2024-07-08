use std::cmp;

use chrono::{Duration, DurationRound, Utc};
use integration_twitch::gql::use_view_count;
use reqwest::Client;
use vtstats_database::{
    stream_stats::AddStreamViewerStatsQuery, streams::get_stream_by_id, PgPool,
};

pub async fn collect_viewers(
    stream_id: i32,
    login: &str,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let st = get_stream_by_id(stream_id, pool).await?;

    let mut last_max = st.as_ref().and_then(|st| st.viewer_max).unwrap_or_default();
    let mut last_avg = st.as_ref().and_then(|st| st.viewer_avg).unwrap_or_default();
    let start = st
        .as_ref()
        .and_then(|st| st.start_time)
        .unwrap_or_else(|| Utc::now().duration_trunc(Duration::seconds(15)).unwrap());

    loop {
        let res = use_view_count(login.to_string(), client).await?;

        if let Some(stream) = res.data.user.stream {
            let time = Utc::now().duration_trunc(Duration::seconds(15))?;

            let count = ((time - start).num_seconds() / 15) as i32;

            let max = cmp::max(last_max, stream.viewers_count);
            let avg = (last_avg * count + stream.viewers_count) / (count + 1);

            AddStreamViewerStatsQuery {
                stream_id,
                time,
                count: stream.viewers_count,
                max,
                avg,
            }
            .execute(pool)
            .await?;

            (last_max, last_avg) = (max, avg);
        }

        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}
