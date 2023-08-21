CREATE TYPE subscription_kind AS ENUM ('telegram_stream_update', 'discord_stream_update');

ALTER TYPE job_kind
ADD
  VALUE 'install_discord_commands';

ALTER TYPE job_kind
ADD
  VALUE 'send_notification';

CREATE TABLE subscriptions (
  subscription_id serial PRIMARY KEY,
  kind subscription_kind NOT NULL,
  payload jsonb NOT NULL DEFAULT 'null',
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE notifications (
  notification_id serial PRIMARY KEY,
  subscription_id integer NOT NULL REFERENCES subscriptions,
  payload jsonb NOT NULL DEFAULT 'null',
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);