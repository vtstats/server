CREATE TABLE channel_revenue_stats (
    channel_id integer NOT NULL REFERENCES channels,
    time timestamptz NOT NULL,
    value JSONB NOT NULL,
    PRIMARY KEY (channel_id, time)
);

ALTER TABLE
    channels
ADD
    COLUMN revenue JSONB NOT NULL DEFAULT 'null',
ADD
    COLUMN revenue_1d_ago JSONB NOT NULL DEFAULT 'null',
ADD
    COLUMN revenue_7d_ago JSONB NOT NULL DEFAULT 'null',
ADD
    COLUMN revenue_30d_ago JSONB NOT NULL DEFAULT 'null';