# Contributing

Thank you for checking out `urashyl-portfolio`!

## Local Development

### Prerequisites

- Rust toolchain (`edition = "2024"`, tested on Rust 1.96.x)
- Zero Node.js or external CSS/JS build tools required — static assets in `static/` are embedded at compile time via `include_str!`.

### Setup

1. Copy `.env.example` to `.env` (`.env` is gitignored).
2. Leave `GRAFANA_API_KEY` as the placeholder to run in standby mode, or point `GRAFANA_URL` + `GRAFANA_API_KEY` at a local/test Grafana instance using a read-only (`Viewer`) Service Account token.
3. Run tests and linter:

```bash
cargo clippy --all-targets
cargo test
```

The test suite includes an embedded mock Grafana HTTP server (`src/grafana.rs`) and full HTTP router/SSE tests (`src/main.rs`), so all tests run locally with no external network dependencies.

## Security & Architecture Rules

- **Strict schedule-only Grafana polling**: Public HTTP routes and SSE streams must only read the cached `TelemetrySnapshot` inside `TelemetryStore`. Never introduce endpoints or query parameters that trigger upstream Grafana API requests on visitor traffic.
- **Privacy & Secret Hygiene**:
  - Never expose `GRAFANA_API_KEY`, private datasource URLs, internal IPv4 addresses, or unlisted container/VM names in JSON responses or HTML templates.
  - Render dynamic SVG and HTML using Maud's `html!` macro or safe DOM `textContent` on the client — never interpolate untrusted remote text into raw HTML.
- **Testing & Code Style**:
  - Run `cargo clippy --all-targets` with zero warnings and `cargo test` before opening a pull request.
  - Keep functions focused and preserve existing unit/integration test coverage for any new behavior.
