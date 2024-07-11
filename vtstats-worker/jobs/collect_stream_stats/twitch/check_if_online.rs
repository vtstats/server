use vtstats_database::{streams::if_stream_is_live, PgPool};

pub async fn check_if_online(stream_id: i32, pool: &PgPool) -> anyhow::Result<()> {
    while if_stream_is_live(stream_id, pool).await? {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }

    Ok(())
}
