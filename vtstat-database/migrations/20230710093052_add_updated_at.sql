ALTER TABLE
    jobs
ADD
    COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();