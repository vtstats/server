ALTER TABLE
    channels
ADD
    COLUMN view integer NOT NULL DEFAULT 0,
ADD
    COLUMN view_1d_ago integer NOT NULL DEFAULT 0,
ADD
    COLUMN view_7d_ago integer NOT NULL DEFAULT 0,
ADD
    COLUMN view_30d_ago integer NOT NULL DEFAULT 0,
ADD
    COLUMN subscriber integer NOT NULL DEFAULT 0,
ADD
    COLUMN subscriber_1d_ago integer NOT NULL DEFAULT 0,
ADD
    COLUMN subscriber_7d_ago integer NOT NULL DEFAULT 0,
ADD
    COLUMN subscriber_30d_ago integer NOT NULL DEFAULT 0;