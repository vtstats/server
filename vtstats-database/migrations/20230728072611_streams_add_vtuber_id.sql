ALTER TABLE
    streams
ADD
    COLUMN vtuber_id TEXT REFERENCES vtubers;

UPDATE
    streams
SET
    vtuber_id = channels.vtuber_id
FROM
    channels
where
    streams.channel_id = channels.channel_id;

ALTER TABLE
    streams
ALTER COLUMN
    vtuber_id
SET
    NOT NULL;