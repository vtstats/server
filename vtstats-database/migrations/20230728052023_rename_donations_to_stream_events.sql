ALTER TYPE donation_kind RENAME TO stream_event_kind;

ALTER TABLE
    donations RENAME TO stream_events;