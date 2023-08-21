-- we only use equal comparisons
CREATE INDEX stream_events_stream_id ON stream_events USING HASH (stream_id);