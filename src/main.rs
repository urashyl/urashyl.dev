mod config;
mod data;
mod grafana;
mod render;

use std::{convert::Infallible, sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{OriginalUri, Path, Query, Request, State},
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{
        Html, IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use futures_util::stream::{self, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    config::AppConfig,
    data::{
        HOMELAB_SERVICES, PROFILE, Project, ProjectCategory, ServiceCategory, adjacent_projects,
        filter_projects, find_project,
    },
    grafana::{TelemetrySnapshot, TelemetryState, TelemetryStore},
};

const STYLES_CSS: &str = include_str!("../static/styles.css");
const APP_JS: &str = include_str!("../static/app.js");
const FAVICON_SVG: &str = include_str!("../static/favicon.svg");

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub telemetry: Arc<TelemetryStore>,
    pub started_at: Instant,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    server_uptime_secs: u64,
    domain: &'static str,
    hypervisor: &'static str,
    grafana_configured: bool,
    grafana_state: TelemetryState,
    grafana_version: Option<String>,
    last_synced_at: Option<String>,
    services_total: usize,
    services_up: usize,
}

#[derive(Serialize)]
struct ServiceWithStatus {
    id: &'static str,
    name: &'static str,
    role: &'static str,
    category: ServiceCategory,
    runtime: &'static str,
    exposure: &'static str,
    description: &'static str,
    highlights: &'static [&'static str],
    up: Option<bool>,
    probe_target: Option<String>,
    probe_source: String,
}

#[derive(Serialize)]
struct SearchResponse {
    query: String,
    results: Vec<SearchHit>,
}

#[derive(Serialize)]
struct SearchHit {
    title: String,
    subtitle: String,
    kind: &'static str,
    href: String,
    external: bool,
}

#[derive(Deserialize, Default)]
struct ProjectsQuery {
    category: Option<String>,
    q: Option<String>,
}

#[derive(Deserialize, Default)]
struct HomelabQuery {
    category: Option<String>,
}

#[derive(Deserialize, Default)]
struct SearchQuery {
    q: Option<String>,
}

async fn security_headers(req: Request, next: Next) -> Response {
    let is_api = req.uri().path().starts_with("/api/");
    let mut res = next.run(req).await;
    let headers = res.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    if is_api && !headers.contains_key(header::CACHE_CONTROL) {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        );
    }
    res
}

async fn home(State(state): State<AppState>) -> Html<String> {
    let snap = state.telemetry.snapshot().await;
    Html(render::render_home(&snap).into_string())
}

async fn projects_page(
    State(state): State<AppState>,
    Query(params): Query<ProjectsQuery>,
) -> Html<String> {
    let snap = state.telemetry.snapshot().await;
    let category = params.category.as_deref().and_then(ProjectCategory::parse);
    let query = params.q.as_deref().unwrap_or("");
    let filtered = filter_projects(category, query);
    Html(render::render_projects(category, query, &filtered, &snap).into_string())
}

async fn project_detail(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Response {
    let snap = state.telemetry.snapshot().await;
    match find_project(&slug) {
        Some(project) => {
            let (prev, next) = adjacent_projects(&slug);
            Html(render::render_project_detail(project, prev, next, &snap).into_string())
                .into_response()
        }
        None => {
            let body = render::render_not_found(&format!("/projects/{slug}"), &snap).into_string();
            (StatusCode::NOT_FOUND, Html(body)).into_response()
        }
    }
}

async fn homelab_page(
    State(state): State<AppState>,
    Query(params): Query<HomelabQuery>,
) -> Html<String> {
    let snap = state.telemetry.snapshot().await;
    let category = params.category.as_deref().and_then(ServiceCategory::parse);
    Html(render::render_homelab(&snap, category).into_string())
}

async fn telemetry_page(State(state): State<AppState>) -> Html<String> {
    let snap = state.telemetry.snapshot().await;
    Html(render::render_telemetry(&snap).into_string())
}

async fn about_page(State(state): State<AppState>) -> Html<String> {
    let snap = state.telemetry.snapshot().await;
    Html(render::render_about(&snap).into_string())
}

async fn not_found_handler(State(state): State<AppState>, OriginalUri(uri): OriginalUri) -> Response {
    let snap = state.telemetry.snapshot().await;
    let html = render::render_not_found(uri.path(), &snap).into_string();
    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

async fn styles() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/css; charset=utf-8"),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=300"),
            ),
        ],
        STYLES_CSS,
    )
}

async fn script() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/javascript; charset=utf-8"),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=300"),
            ),
        ],
        APP_JS,
    )
}

async fn favicon() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("image/svg+xml"),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=300"),
            ),
        ],
        FAVICON_SVG,
    )
}

async fn api_health(State(state): State<AppState>) -> Json<HealthResponse> {
    let snap = state.telemetry.snapshot().await;
    let services_up = snap
        .service_probes
        .iter()
        .filter(|p| p.up == Some(true))
        .count();

    Json(HealthResponse {
        status: "ok",
        server_uptime_secs: state.started_at.elapsed().as_secs(),
        domain: PROFILE.domain,
        hypervisor: PROFILE.hypervisor,
        grafana_configured: snap.configured,
        grafana_state: snap.state,
        grafana_version: snap.grafana_version,
        last_synced_at: snap.last_synced_at,
        services_total: HOMELAB_SERVICES.len(),
        services_up,
    })
}

async fn api_telemetry(State(state): State<AppState>) -> Json<TelemetrySnapshot> {
    Json(state.telemetry.snapshot().await)
}

async fn api_telemetry_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let initial_snap = state.telemetry.snapshot().await;
    let rx = state.telemetry.subscribe();

    let initial_event = serde_json::to_string(&initial_snap)
        .ok()
        .map(|json| Ok::<Event, Infallible>(Event::default().event("telemetry").data(json)));

    let updates = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(snap) => {
                    if let Ok(json) = serde_json::to_string(&snap) {
                        return Some((Ok(Event::default().event("telemetry").data(json)), rx));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    let combined = stream::iter(initial_event).chain(updates);
    Sse::new(combined).keep_alive(KeepAlive::default())
}

async fn api_projects(Query(params): Query<ProjectsQuery>) -> Json<Vec<&'static Project>> {
    let category = params.category.as_deref().and_then(ProjectCategory::parse);
    let query = params.q.as_deref().unwrap_or("");
    Json(filter_projects(category, query))
}

async fn api_project_by_slug(Path(slug): Path<String>) -> Response {
    match find_project(&slug) {
        Some(project) => Json(project).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "project not found" })),
        )
            .into_response(),
    }
}

async fn api_services(State(state): State<AppState>) -> Json<Vec<ServiceWithStatus>> {
    let snap = state.telemetry.snapshot().await;
    let merged = HOMELAB_SERVICES
        .iter()
        .map(|svc| {
            let probe = snap.probe_for_service(svc.id);
            ServiceWithStatus {
                id: svc.id,
                name: svc.name,
                role: svc.role,
                category: svc.category,
                runtime: svc.runtime,
                exposure: svc.exposure,
                description: svc.description,
                highlights: svc.highlights,
                up: probe.and_then(|p| p.up),
                probe_target: probe.and_then(|p| p.target.clone()),
                probe_source: probe
                    .map(|p| p.source.clone())
                    .unwrap_or_else(|| "unconfigured".to_string()),
            }
        })
        .collect();

    Json(merged)
}

async fn api_search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Json<SearchResponse> {
    let raw_q = params.q.unwrap_or_default();
    let q = raw_q.trim().to_ascii_lowercase();
    let snap = state.telemetry.snapshot().await;
    let mut results = Vec::new();

    let pages = [
        ("Overview", "Landing summary & live telemetry panel", "/"),
        ("Projects", "All engineering repositories & case studies", "/projects"),
        ("Homelab", "Proxmox VE topology & service matrix", "/homelab"),
        ("Telemetry", "Live Grafana metrics, alerts & setup", "/telemetry"),
        ("About", "Profile, principles, stack & colophon", "/about"),
    ];
    for (title, sub, href) in pages {
        if q.is_empty()
            || title.to_ascii_lowercase().contains(&q)
            || sub.to_ascii_lowercase().contains(&q)
        {
            results.push(SearchHit {
                title: title.to_string(),
                subtitle: sub.to_string(),
                kind: "page",
                href: href.to_string(),
                external: false,
            });
        }
    }

    for project in filter_projects(None, &q) {
        results.push(SearchHit {
            title: project.name.to_string(),
            subtitle: format!("{} · {}", project.language, project.tagline),
            kind: "project",
            href: format!("/projects/{}", project.id),
            external: false,
        });
    }

    for svc in HOMELAB_SERVICES {
        if q.is_empty()
            || svc.name.to_ascii_lowercase().contains(&q)
            || svc.role.to_ascii_lowercase().contains(&q)
            || svc.description.to_ascii_lowercase().contains(&q)
            || svc.runtime.to_ascii_lowercase().contains(&q)
        {
            results.push(SearchHit {
                title: svc.name.to_string(),
                subtitle: format!("{} · {}", svc.role, svc.runtime),
                kind: "service",
                href: format!("/homelab?category={}", svc.category.as_str()),
                external: false,
            });
        }
    }

    for dash in &snap.dashboards {
        if q.is_empty()
            || dash.title.to_ascii_lowercase().contains(&q)
            || dash
                .tags
                .iter()
                .any(|t| t.to_ascii_lowercase().contains(&q))
        {
            let (href, external) = match &dash.url {
                Some(url) => (url.clone(), true),
                None => ("/telemetry".to_string(), false),
            };
            results.push(SearchHit {
                title: dash.title.clone(),
                subtitle: dash
                    .folder_title
                    .clone()
                    .unwrap_or_else(|| "Grafana dashboard".to_string()),
                kind: "dashboard",
                href,
                external,
            });
        }
    }

    for alert in &snap.alerts {
        if q.is_empty()
            || alert.name.to_ascii_lowercase().contains(&q)
            || alert
                .summary
                .as_deref()
                .unwrap_or("")
                .to_ascii_lowercase()
                .contains(&q)
        {
            results.push(SearchHit {
                title: alert.name.clone(),
                subtitle: format!("{} ({})", alert.severity, alert.state),
                kind: "alert",
                href: "/telemetry".to_string(),
                external: false,
            });
        }
    }

    results.truncate(16);
    Json(SearchResponse {
        query: raw_q,
        results,
    })
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/projects", get(projects_page))
        .route("/projects/{slug}", get(project_detail))
        .route("/homelab", get(homelab_page))
        .route("/telemetry", get(telemetry_page))
        .route("/about", get(about_page))
        .route("/styles.css", get(styles))
        .route("/app.js", get(script))
        .route("/favicon.svg", get(favicon))
        .route("/api/health", get(api_health))
        .route("/api/telemetry", get(api_telemetry))
        .route("/api/telemetry/stream", get(api_telemetry_stream))
        .route("/api/projects", get(api_projects))
        .route("/api/projects/{slug}", get(api_project_by_slug))
        .route("/api/services", get(api_services))
        .route("/api/search", get(api_search))
        .fallback(not_found_handler)
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "urashyl_portfolio=info".into()),
        )
        .init();

    let config = Arc::new(AppConfig::load());
    let telemetry = TelemetryStore::new(config.clone());
    telemetry.clone().spawn_poller();

    let bind_addr = config.bind_addr;
    let state = AppState {
        config,
        telemetry,
        started_at: Instant::now(),
    };

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("failed to bind TCP listener");

    info!(%bind_addr, "urashyl.dev listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    async fn spawn_test_server() -> std::net::SocketAddr {
        let cfg = Arc::new(AppConfig::from_lookup(|k| {
            if k == "BIND_ADDR" {
                Some("127.0.0.1:0".to_string())
            } else {
                None
            }
        }));
        let telemetry = TelemetryStore::new(cfg.clone());
        let state = AppState {
            config: cfg,
            telemetry,
            started_at: Instant::now(),
        };
        let app = build_router(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        addr
    }

    #[tokio::test]
    async fn serves_multipage_html_assets_and_404_with_security_headers() {
        let addr = spawn_test_server().await;
        let client = reqwest::Client::new();
        let base = format!("http://{addr}");

        for (path, expected) in [
            ("/", "Live Cluster Telemetry"),
            ("/projects", "Projects &amp; Architectural Case Studies"),
            ("/projects?category=systems", "mitch.pro"),
            ("/projects/mitch-pro", "Component Layers"),
            ("/homelab", "Proxmox VE Homelab"),
            ("/telemetry", "Live Grafana Telemetry"),
            ("/about", "Technical Stack Matrix"),
        ] {
            let resp = client.get(format!("{base}{path}")).send().await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK, "GET {path}");
            assert_eq!(
                resp.headers()
                    .get("x-content-type-options")
                    .and_then(|v| v.to_str().ok()),
                Some("nosniff")
            );
            assert_eq!(
                resp.headers()
                    .get("x-frame-options")
                    .and_then(|v| v.to_str().ok()),
                Some("DENY")
            );
            let body = resp.text().await.unwrap();
            assert!(
                body.contains(expected),
                "expected '{expected}' in {path}"
            );
        }

        // 404 HTML routes (including removed repositories)
        for bad_path in [
            "/projects/does-not-exist",
            "/projects/calc",
            "/projects/chadwm",
            "/projects/dotfiles",
            "/unknown-page",
        ] {
            let resp = client
                .get(format!("{base}{bad_path}"))
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let body = resp.text().await.unwrap();
            assert!(body.contains("HTTP 404"));
        }

        // Static assets
        for (asset, ctype) in [
            ("/styles.css", "text/css"),
            ("/app.js", "application/javascript"),
            ("/favicon.svg", "image/svg+xml"),
        ] {
            let resp = client.get(format!("{base}{asset}")).send().await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let ct = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert!(ct.contains(ctype), "asset {asset} content-type {ct}");
        }
    }

    #[tokio::test]
    async fn serves_json_apis_search_and_refresh() {
        let addr = spawn_test_server().await;
        let client = reqwest::Client::new();
        let base = format!("http://{addr}");

        let health = client.get(format!("{base}/api/health")).send().await.unwrap();
        assert_eq!(health.status(), StatusCode::OK);
        assert_eq!(
            health
                .headers()
                .get(reqwest::header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("no-store")
        );
        let health_json: serde_json::Value = health.json().await.unwrap();
        assert_eq!(health_json["status"], "ok");
        assert_eq!(health_json["grafana_state"], "unconfigured");

        let telem: serde_json::Value = client
            .get(format!("{base}/api/telemetry"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(telem["state"], "unconfigured");
        assert_eq!(telem["configured"], false);

        let all_projects: Vec<serde_json::Value> = client
            .get(format!("{base}/api/projects"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(all_projects.len(), 4);

        let projects: Vec<serde_json::Value> = client
            .get(format!("{base}/api/projects?category=systems&q=axum"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0]["id"], "mitch-pro");

        let lacuna = client
            .get(format!("{base}/api/projects/lacuna"))
            .send()
            .await
            .unwrap();
        assert_eq!(lacuna.status(), StatusCode::OK);

        let missing = client
            .get(format!("{base}/api/projects/nonexistent"))
            .send()
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);

        let services: Vec<serde_json::Value> = client
            .get(format!("{base}/api/services"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(services.len(), 6);

        let search: serde_json::Value = client
            .get(format!("{base}/api/search?q=proxmox"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(!search["results"].as_array().unwrap().is_empty());

        let refresh_resp = client
            .post(format!("{base}/api/telemetry/refresh"))
            .send()
            .await
            .unwrap();
        assert!(
            refresh_resp.status() == StatusCode::NOT_FOUND
                || refresh_resp.status() == StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn sse_stream_emits_initial_telemetry_event() {
        let addr = spawn_test_server().await;
        let client = reqwest::Client::new();
        let mut resp = client
            .get(format!("http://{addr}/api/telemetry/stream"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ctype = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ctype.contains("text/event-stream"), "unexpected ctype: {ctype}");

        let chunk = tokio::time::timeout(std::time::Duration::from_secs(3), resp.chunk())
            .await
            .expect("SSE initial chunk timed out")
            .unwrap()
            .expect("expected initial SSE chunk");
        let text = String::from_utf8(chunk.to_vec()).unwrap();
        assert!(text.contains("event: telemetry"), "SSE payload: {text}");
        let data_line = text
            .lines()
            .find_map(|l| l.strip_prefix("data:"))
            .map(str::trim)
            .expect("expected data line in SSE event");
        let parsed: serde_json::Value = serde_json::from_str(data_line).unwrap();
        assert_eq!(parsed["state"], "unconfigured");
    }
}
