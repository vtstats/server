ALTER TABLE
    channel_stats_summary
ADD
    COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();