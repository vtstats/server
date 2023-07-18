ALTER TABLE
    subscriptions
ADD
    CONSTRAINT subscriptions_kind_payload_key UNIQUE (kind, payload);