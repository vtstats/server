use vtstats_database::{
    streams::{find_stream, StreamStatus},
    PgPool,
};

pub async fn check_if_online(stream_id: i32, pool: &PgPool) -> anyhow::Result<()> {
    loop {
        let stream = find_stream(stream_id, pool).await?;

        if !matches!(stream, Some(stream) if stream.status == StreamStatus::Live) {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
