# urashyl.dev

Open-source, single-binary Rust + Axum portfolio and live Proxmox VE homelab observability portal for [urashyl](https://github.com/urashyl) (`urashyl.dev`).

## Architecture & Pages

- **Zero Node.js Runtime** — Pure Rust (`axum 0.8`, `tokio`, `maud`, `reqwest` over `rustls-tls`), with CSS, JavaScript, and SVG assets embedded directly via `include_str!`.
- **Strict Cache-Only Grafana Polling (Abuse-Resistant)** — Only the server's internal background poller (`GRAFANA_POLL_INTERVAL_SECS`) queries Grafana. Visitor page loads, JSON API requests, and Server-Sent Events read exclusively from the sanitized in-memory cache; no public endpoint can trigger upstream Grafana queries.
- **Multi-Page Routes**:
  - `/` — Intro, a live CPU trace from the homelab, selected work, and what runs at home.
  - `/projects` — All projects, filterable by category (`?category=systems|desktop`, `?q=`).
  - `/projects/{slug}` — Write-ups (`mitch-pro`, `lacuna`, `ai-container`, `niri`): how each is built and why.
  - `/homelab` — Topology diagram and the services on the Proxmox host, with live up/down status.
  - `/telemetry` — Server status: CPU/memory/disk/guests/network charts, service checks, alerts, and events, updated over SSE.
  - `/about` — Background, principles, tools, and how the site works.
  - `/robots.txt`, `/sitemap.xml`. CSS/JS/favicon URLs carry a content hash (`?v=`) and are cached as immutable.
- **Read-Only JSON & Streaming APIs**:
  - `GET /api/health` — Server status, uptime, and cached Grafana connection state.
  - `GET /api/telemetry` — Current sanitized Grafana telemetry snapshot from cache.
  - `GET /api/telemetry/stream` — Real-time Server-Sent Events (`event: telemetry`) pushing cached updates when the background poller ticks.
  - `GET /api/projects` & `GET /api/projects/{slug}` — Project catalog and case study JSON.
  - `GET /api/services` — Homelab service matrix merged with cached Grafana service status.
  - `GET /api/search?q=` — Global search powering the `⌘K` / `/` command palette.

## Grafana API Setup & Privacy Sanitization

Your `GRAFANA_API_KEY` stays strictly server-side; the browser only receives sanitized metrics and metadata. Internal IPv4 addresses (`x.x.x.x`), internal `instance` / `id` alert labels, and unlisted private Proxmox VMs/LXCs are stripped automatically before caching.

1. In Grafana, open **Administration → Users and access → Service accounts**.
2. Create a service account (e.g. `urashyl-portfolio`) with the **Viewer** role.
3. Click **Add service account token** and copy the `glsa_...` token.
4. Copy `.env.example` to `.env` (already gitignored) and configure your instance:

```dotenv
BIND_ADDR=0.0.0.0:3000
GRAFANA_URL=https://grafana.urashyl.dev
GRAFANA_API_KEY=glsa_your_service_account_token_here
```

Default Prometheus queries work out of the box with both **Proxmox `prometheus-pve-exporter`** (`pve_*` metrics + active guest count + `pve_up` service matching) and standard Linux **`node_exporter`** / **`blackbox_exporter`**, as well as **Proxmox InfluxDB** (`GRAFANA_DATASOURCE_TYPE=influxdb`).

### Supported Environment Variables

| Variable | Default | Notes |
| --- | --- | --- |
| `BIND_ADDR` | `0.0.0.0:3000` | Socket bind address |
| `RUST_LOG` | `urashyl_portfolio=info` | Tracing filter |
| `GRAFANA_URL` | — | Base URL of your Grafana server (required to enable background polling) |
| `GRAFANA_API_KEY` | — | Grafana Service Account token (required to enable background polling) |
| `GRAFANA_PUBLIC_URL` | — | Optional public URL used to render clickable dashboard links |
| `GRAFANA_EXPOSE_LINKS` | `false` | Set `true` to link dashboards using `GRAFANA_URL` when `GRAFANA_PUBLIC_URL` is unset |
| `GRAFANA_ORG_ID` | — | Optional Grafana organization ID (`X-Grafana-Org-Id`) |
| `GRAFANA_DATASOURCE_UID` | — | Optional explicit datasource UID (auto-discovers Prometheus/VictoriaMetrics/InfluxDB otherwise) |
| `GRAFANA_DATASOURCE_TYPE` | `prometheus` | `prometheus` (supports `pve-exporter` + `node_exporter`) or `influxdb` |
| `GRAFANA_DASHBOARD_TAG` | — | Optional tag filter for `/api/search` dashboard indexing |
| `GRAFANA_POLL_INTERVAL_SECS` | `30` | Background polling interval (`10`–`3600` seconds) |
| `GRAFANA_TIMEOUT_SECS` | `8` | HTTP client timeout (`2`–`30` seconds) |
| `GRAFANA_TIME_RANGE` | `6h` | Query window (`1h`, `3h`, `6h`, `12h`, `24h`, `7d`) |
| `GRAFANA_INSECURE_TLS` | `false` | Allow self-signed TLS certificates for internal Grafana instances |
| `GRAFANA_QUERY_*` | built-in | Override `CPU`, `MEMORY`, `DISK`, `LOAD`, `GUESTS`, `UPTIME`, `NETWORK`, or `SERVICES` queries |

## Build & Run

```bash
# Run unit and integration tests
cargo test

# Run in release mode
cargo run --release
```

## Running as a service

The repo is meant to live at `~/urashyl.dev` and run as a systemd user service (`deploy/urashyl-dev.service`). The binary reads `.env` from that directory, so `BIND_ADDR` and the Grafana settings go there.

```bash
mv ~/portfolio ~/urashyl.dev     # once, if it's checked out elsewhere
~/urashyl.dev/deploy/deploy.sh   # build, install the unit, enable and restart it
sudo loginctl enable-linger "$USER"   # once: keep it running after logout and start at boot
```

Re-run `deploy/deploy.sh` after pulling changes. Logs: `journalctl --user -u urashyl-dev -f`.

## Open Source & License

Released under the [MIT License](LICENSE). Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).
