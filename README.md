# vtstats server

## Deploy

We use podman to simplify the deployment.

First, put these files on your linux machine:

<details>
<summary>
<code>/etc/containers/systemd/vtstats-database.container</code>
</summary>

```ini
[Unit]
Description=vtstats postgresql database

[Container]
Image=docker.io/groonga/pgroonga:latest-debian-15
Exec=-c shared_preload_libraries='pg_stat_statements' -c pg_stat_statements.max=10000 -c pg_stat_statements.track=all
ContainerName=vtstats-database
IP=10.88.0.10
AutoUpdate=local
Network=podman
LogDriver=journald
EnvironmentFile=/path/to/your/vtstats.env
Volume=/path/to/your/postgres:/var/lib/postgresql/data

[Service]
Restart=always
TimeoutStartSec=900

[Install]
WantedBy=multi-user.target default.target
```

</details>

<details>
<summary>
<code>/etc/containers/systemd/vtstats-api.container</code>
</summary>

```ini
[Container]
Image=ghcr.io/vtstats/server:latest
Exec=api
ContainerName=vtstats-api
IP=10.88.0.11
AutoUpdate=registry
Network=podman
LogDriver=journald
EnvironmentFile=/path/to/vtstats.env

[Service]
Restart=always
TimeoutStartSec=900

[Install]
WantedBy=multi-user.target default.target
```

</details>

<details>
<summary>
<code>/etc/containers/systemd/vtstats-worker.container</code>
</summary>

```ini
[Container]
Image=ghcr.io/vtstats/server:latest
Exec=worker
ContainerName=vtstats-worker
IP=10.88.0.12
AutoUpdate=registry
Network=podman
LogDriver=journald
EnvironmentFile=/path/to/vtstats.env

[Service]
Restart=always
TimeoutStartSec=900

[Install]
WantedBy=multi-user.target default.target
```

</details>

<details>
<summary>
<code>/path/to/vtstats.env</code>
</summary>

```env
# server
SERVER_HOSTNAME=
SERVER_ADDRESS=

# database
POSTGRES_PASSWORD=
POSTGRES_USER=
POSTGRES_DB=
DATABASE_URL=

# youtube
YOUTUBE_PUBSUB_SECRET=
YOUTUBE_API_KEYS=
INNERTUBE_API_KEY=
INNERTUBE_CLIENT_NAME=
INNERTUBE_CLIENT_VERSION=

# bilibili
BILIBILI_COOKIE=

# telegram
TELEGRAM_BOT_TOKEN=
TELEGRAM_SECRET_TOKEN=

# discord
DISCORD_APPLICATION_ID=
DISCORD_APPLICATION_PUBLIC_KEY=
DISCORD_BOT_TOKEN=

# s3
S3_PUBLIC_URL=
S3_HOST=
S3_REGION=
S3_BUCKET=
S3_KEY_ID=
S3_ACCESS_KEY=

# admin api
ADMIN_USER_EMAIL=
GOOGLE_CLIENT_ID=

# twitch
TWITCH_CLIENT_ID=
TWITCH_CLIENT_SECRET=
TWITCH_WEBHOOK_SECRET=
```

</details>

Then start the database and run migration

```shell
sudo systemctl daemon-reload
sudo systemctl start vtstats-database
sudo podman run -it -e DATABASE_URL=YOUR_DATABASE_URL ghcr.io/vtstats/server:latest database-migrate
```

Finally, start the api server and job worker:

```shell
sudo systemctl start vtstats-api
sudo systemctl start vtstats-worker

# check status
sudo systemctl status 'vtstats-*'

# view log
sudo journalctl -u vtstats-api --since '1 min ago' -f
```

## Update

```shell
sudo podman pull ghcr.io/vtstats/server:latest
sudo podman run -it -e DATABASE_URL=YOUR_DATABASE_URL ghcr.io/vtstats/server:latest database-migrate
sudo podman auto-update
```

## Grafana

<details>
<summary>
<code>/etc/grafana-agent.yaml</code>
</summary>

```yaml
server:
  log_level: info

metrics:
  wal_directory: /tmp/agent
  global:
    scrape_interval: 60s
    remote_write:
      - url: your-url
  configs:
    - name: web
      scrape_configs:
        - job_name: web
          static_configs:
            - targets: ["10.88.0.11:9000"]
    - name: worker
      scrape_configs:
        - job_name: worker
          static_configs:
            - targets: ["10.88.0.12:9000"]

logs:
  configs:
    - name: journal
      clients:
        - url: your-url
      positions:
        filename: /tmp/agent/positions.yaml
      scrape_configs:
        - job_name: journal
          journal:
            json: false
            max_age: 6h
            path: /var/log/journal
          relabel_configs:
            - action: keep
              source_labels: ["__journal__systemd_unit"]
              regex: .*vtstats.*
            - source_labels: ["__journal__systemd_unit"]
              target_label: unit
          pipeline_stages:
            - json:
                drop_malformed: true
                expressions:
                  level: level
                  timestamp: timestamp
                  message: message
            - timestamp:
                source: timestamp
                format: RFC3339Nano
            - labels:
                level:
            - output:
                source: message

integrations:
  node_exporter:
    enabled: true
```

</details>
