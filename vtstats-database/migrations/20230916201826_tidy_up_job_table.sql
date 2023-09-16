ALTER TYPE job_kind RENAME VALUE 'update_youtube_channel_view_and_subscriber' TO 'update_channel_stats';

DELETE FROM
    jobs
WHERE
    kind = 'update_bilibili_channel_view_and_subscriber'
    OR kind = 'update_youtube_channel_donation'
    OR kind = 'update_currency_exchange_rate'
    OR kind = 'upsert_youtube_stream'
    OR kind = 'update_upcoming_stream'
    OR kind = 'collect_youtube_stream_live_chat'
    OR kind = 'install_discord_commands';

UPDATE
    jobs
SET
    payload = json_build_object('stream_id', (payload ->> 'stream_id') :: int)
WHERE
    kind = 'collect_youtube_stream_metadata'
    OR kind = 'collect_twitch_stream_metadata';

DELETE FROM
    jobs
WHERE
    kind = 'send_notification'
    AND (payload ->> 'platform') != 'youtube';

DELETE FROM
    jobs
WHERE
    kind = 'send_notification'
    AND (payload ->> 'stream_platform_id') NOT IN (
        SELECT
            platform_id
        FROM
            streams
    );

UPDATE
    jobs
SET
    payload = json_build_object(
        'stream_id',
        (
            SELECT
                stream_id
            FROM
                streams
            WHERE
                platform_id = (payload ->> 'stream_platform_id')
        )
    )
WHERE
    kind = 'send_notification'
    AND (payload ->> 'stream_platform_id') IS NOT NULL;