CREATE TYPE channel_stats_kind AS ENUM ('subscriber', 'view', 'revenue');

CREATE TABLE channel_stats_summary (
    channel_id integer NOT NULL REFERENCES channels,
    kind channel_stats_kind NOT NULL,
    value JSONB NOT NULL,
    value_1_day_ago JSONB NOT NULL,
    value_7_days_ago JSONB NOT NULL,
    value_30_days_ago JSONB NOT NULL,
    PRIMARY KEY (channel_id, kind)
);

INSERT INTO
    channel_stats_summary (
        channel_id,
        kind,
        value,
        value_1_day_ago,
        value_7_days_ago,
        value_30_days_ago
    )
SELECT
    channel_id,
    'subscriber' kind,
    to_jsonb(subscriber) value,
    to_jsonb(subscriber_1d_ago) value_1_day_ago,
    to_jsonb(subscriber_7d_ago) value_7_days_ago,
    to_jsonb(subscriber_30d_ago) value_30_days_ago
FROM
    channels;

INSERT INTO
    channel_stats_summary (
        channel_id,
        kind,
        value,
        value_1_day_ago,
        value_7_days_ago,
        value_30_days_ago
    )
SELECT
    channel_id,
    'revenue' kind,
    revenue value,
    revenue_1d_ago value_1_day_ago,
    revenue_7d_ago value_7_days_ago,
    revenue_30d_ago value_30_days_ago
FROM
    channels
WHERE
    platform = 'youtube'
    OR platform = 'twitch';

INSERT INTO
    channel_stats_summary (
        channel_id,
        kind,
        value,
        value_1_day_ago,
        value_7_days_ago,
        value_30_days_ago
    )
SELECT
    channel_id,
    'view' kind,
    to_jsonb(view) value,
    to_jsonb(view_1d_ago) value_1_day_ago,
    to_jsonb(view_7d_ago) value_7_days_ago,
    to_jsonb(view_30d_ago) value_30_days_ago
FROM
    channels
WHERE
    platform = 'youtube'
    OR platform = 'bilibili';

ALTER TABLE
    channels DROP COLUMN view,
    DROP COLUMN view_1d_ago,
    DROP COLUMN view_7d_ago,
    DROP COLUMN view_30d_ago,
    DROP COLUMN subscriber,
    DROP COLUMN subscriber_1d_ago,
    DROP COLUMN subscriber_7d_ago,
    DROP COLUMN subscriber_30d_ago,
    DROP COLUMN revenue,
    DROP COLUMN revenue_1d_ago,
    DROP COLUMN revenue_7d_ago,
    DROP COLUMN revenue_30d_ago;