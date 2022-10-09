CREATE TABLE vtubers (
  vtuber_id text PRIMARY KEY,
  native_name text NOT NULL,
  english_name text,
  japanese_name text,
  twitter_username text,
  debuted_at timestamptz,
  retired_at timestamptz
);

CREATE TYPE group_kind AS ENUM ('agency', 'branch', 'individual');

CREATE TABLE groups (
  group_id text PRIMARY KEY,
  group_type group_kind NOT NULL,
  native_name text NOT NULL,
  english_name text,
  japanese_name text,
  romaji_name text,
  created_at timestamptz,
  children text []
);

CREATE TYPE platform AS ENUM ('youtube', 'bilibili', 'twitch');

CREATE TABLE channels (
  channel_id serial PRIMARY KEY,
  platform platform NOT NULL,
  platform_id text NOT NULL,
  kind text NOT NULL,
  vtuber_id text NOT NULL REFERENCES vtubers
);

CREATE UNIQUE INDEX channels_platform_id ON channels (platform, platform_id);

-- update once each hour
CREATE TABLE channel_view_stats (
  channel_id integer NOT NULL REFERENCES channels,
  time timestamptz NOT NULL,
  count integer NOT NULL,
  PRIMARY KEY (channel_id, time)
);

-- update once each hour
CREATE TABLE channel_subscriber_stats (
  channel_id integer NOT NULL REFERENCES channels,
  time timestamptz NOT NULL,
  count integer NOT NULL,
  PRIMARY KEY (channel_id, time)
);

-- update once each day
CREATE TABLE channel_donation_stats (
  channel_id integer NOT NULL REFERENCES channels,
  time timestamptz NOT NULL,
  sum jsonb NOT NULL,
  PRIMARY KEY (channel_id, time)
);

CREATE TYPE stream_status AS ENUM ('scheduled', 'live', 'ended');

CREATE TABLE streams (
  stream_id serial PRIMARY KEY,
  channel_id integer NOT NULL REFERENCES channels,
  platform platform NOT NULL,
  platform_id text NOT NULL,
  title text NOT NULL,
  thumbnail_url text,
  schedule_time timestamptz,
  start_time timestamptz,
  end_time timestamptz,
  status stream_status NOT NULL,
  viewer_max integer,
  viewer_avg integer,
  like_max integer,
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX streams_platform_id ON streams (platform, platform_id);

-- update once 15 seconds
CREATE TABLE stream_viewer_stats (
  stream_id integer NOT NULL REFERENCES streams,
  time timestamptz NOT NULL,
  count integer NOT NULL,
  PRIMARY KEY (stream_id, time)
);

-- update once 15 seconds
CREATE TABLE stream_chat_stats (
  stream_id integer NOT NULL REFERENCES streams,
  time timestamptz NOT NULL,
  count integer NOT NULL,
  from_member_count integer NOT NULL,
  PRIMARY KEY (stream_id, time)
);

CREATE TYPE donation_kind AS ENUM (
  'youtube_super_chat',
  'youtube_super_sticker',
  'youtube_new_member',
  'youtube_member_milestone'
);

CREATE TABLE donations (
  stream_id integer NOT NULL REFERENCES streams,
  time timestamptz NOT NULL,
  kind donation_kind NOT NULL,
  value jsonb NOT NULL
);

-- exchange rate update each day
CREATE TABLE currencies (
  code text NOT NULL PRIMARY KEY,
  symbol text NOT NULL,
  rate REAL NOT NULL,
  updated_at timestamptz NOT NULL
);

CREATE TYPE job_status AS ENUM (
  'queued',
  'running',
  'success',
  'failed'
);

CREATE TYPE job_kind AS ENUM (
  'health_check',
  'refresh_youtube_rss',
  'subscribe_youtube_pubsub',
  'update_youtube_channel_view_and_subscriber',
  'update_bilibili_channel_view_and_subscriber',
  'update_youtube_channel_donation',
  'update_currency_exchange_rate',
  'upsert_youtube_stream',
  'collect_youtube_stream_metadata',
  'collect_youtube_stream_live_chat'
);

CREATE TABLE jobs (
  job_id serial NOT NULL,
  kind job_kind NOT NULL,
  payload jsonb NOT NULL,
  status job_status NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  next_run timestamptz,
  last_run timestamptz,
  continuation text,
  UNIQUE (kind, payload)
);