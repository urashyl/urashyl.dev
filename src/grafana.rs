use std::{
    collections::HashMap,
    net::Ipv4Addr,
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info, warn};

use crate::{
    config::{AppConfig, GrafanaConfig},
    data::HOMELAB_SERVICES,
};

const MAX_CHART_POINTS: usize = 72;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TelemetryState {
    Unconfigured,
    Connected,
    Degraded,
    Error,
}

impl TelemetryState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unconfigured => "Standby",
            Self::Connected => "Live",
            Self::Degraded => "Degraded",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TelemetryPoint {
    pub ts_ms: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricSeries {
    pub key: String,
    pub label: String,
    pub unit: String,
    pub current: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub avg: Option<f64>,
    pub points: Vec<TelemetryPoint>,
    pub query_error: Option<String>,
}

impl MetricSeries {
    pub fn empty(key: &str, label: &str, unit: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            unit: unit.to_string(),
            current: None,
            min: None,
            max: None,
            avg: None,
            points: Vec::new(),
            query_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClusterMetrics {
    pub cpu: MetricSeries,
    pub memory: MetricSeries,
    pub disk: MetricSeries,
    pub load: MetricSeries,
    pub network: MetricSeries,
    pub uptime_seconds: Option<f64>,
}

impl Default for ClusterMetrics {
    fn default() -> Self {
        Self {
            cpu: MetricSeries::empty("cpu", "CPU Utilization", "%"),
            memory: MetricSeries::empty("memory", "Memory Utilization", "%"),
            disk: MetricSeries::empty("disk", "Disk Utilization", "%"),
            load: MetricSeries::empty("load", "Load Average (1m)", ""),
            network: MetricSeries::empty("network", "Network Throughput", "MB/s"),
            uptime_seconds: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GrafanaDatasourceSummary {
    pub uid: String,
    pub name: String,
    pub ds_type: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GrafanaDashboardSummary {
    pub uid: String,
    pub title: String,
    pub folder_title: Option<String>,
    pub tags: Vec<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GrafanaAlert {
    pub name: String,
    pub state: String,
    pub severity: String,
    pub summary: Option<String>,
    pub starts_at: Option<String>,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GrafanaAnnotation {
    pub id: i64,
    pub text: String,
    pub tags: Vec<String>,
    pub time_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceProbeStatus {
    pub service_id: String,
    pub name: String,
    pub up: Option<bool>,
    pub target: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetrySnapshot {
    pub state: TelemetryState,
    pub configured: bool,
    pub grafana_version: Option<String>,
    pub grafana_database: Option<String>,
    pub org_name: Option<String>,
    pub active_datasource: Option<GrafanaDatasourceSummary>,
    pub datasources: Vec<GrafanaDatasourceSummary>,
    pub dashboards: Vec<GrafanaDashboardSummary>,
    pub alerts: Vec<GrafanaAlert>,
    pub annotations: Vec<GrafanaAnnotation>,
    pub metrics: ClusterMetrics,
    pub service_probes: Vec<ServiceProbeStatus>,
    pub time_range: String,
    pub poll_interval_secs: u64,
    pub last_synced_at: Option<String>,
    pub last_error: Option<String>,
    pub diagnostics: Vec<String>,
}

impl TelemetrySnapshot {
    pub fn unconfigured(config: &AppConfig) -> Self {
        Self {
            state: TelemetryState::Unconfigured,
            configured: false,
            grafana_version: None,
            grafana_database: None,
            org_name: None,
            active_datasource: None,
            datasources: Vec::new(),
            dashboards: Vec::new(),
            alerts: Vec::new(),
            annotations: Vec::new(),
            metrics: ClusterMetrics::default(),
            service_probes: default_unmonitored_probes(),
            time_range: config.grafana_time_range.clone(),
            poll_interval_secs: config.grafana_poll_interval_secs,
            last_synced_at: None,
            last_error: None,
            diagnostics: vec![
                "Grafana telemetry is in standby. Set GRAFANA_URL and GRAFANA_API_KEY in `.env` or process environment variables to enable live metrics.".to_string(),
            ],
        }
    }

    pub fn pending(config: &AppConfig) -> Self {
        Self {
            state: TelemetryState::Degraded,
            configured: true,
            grafana_version: None,
            grafana_database: None,
            org_name: None,
            active_datasource: None,
            datasources: Vec::new(),
            dashboards: Vec::new(),
            alerts: Vec::new(),
            annotations: Vec::new(),
            metrics: ClusterMetrics::default(),
            service_probes: default_unmonitored_probes(),
            time_range: config.grafana_time_range.clone(),
            poll_interval_secs: config.grafana_poll_interval_secs,
            last_synced_at: None,
            last_error: None,
            diagnostics: vec!["Initial Grafana synchronization queued...".to_string()],
        }
    }

    pub fn probe_for_service(&self, service_id: &str) -> Option<&ServiceProbeStatus> {
        self.service_probes
            .iter()
            .find(|p| p.service_id == service_id)
    }

    pub fn firing_alerts_count(&self) -> usize {
        self.alerts
            .iter()
            .filter(|a| !a.state.eq_ignore_ascii_case("suppressed"))
            .count()
    }
}

fn default_unmonitored_probes() -> Vec<ServiceProbeStatus> {
    HOMELAB_SERVICES
        .iter()
        .map(|svc| ServiceProbeStatus {
            service_id: svc.id.to_string(),
            name: svc.name.to_string(),
            up: None,
            target: None,
            source: "unconfigured".to_string(),
        })
        .collect()
}

pub struct GrafanaClient {
    config: GrafanaConfig,
    http: reqwest::Client,
}

impl GrafanaClient {
    pub fn new(config: GrafanaConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(config.insecure_tls)
            .user_agent("urashyl-portfolio/0.2")
            .build()
            .map_err(|e| format!("failed to build Grafana HTTP client: {e}"))?;
        Ok(Self { config, http })
    }

    fn build_get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.url, path);
        let mut req = self
            .http
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.config.api_key),
            )
            .header(reqwest::header::ACCEPT, "application/json");
        if let Some(org_id) = self.config.org_id {
            req = req.header("X-Grafana-Org-Id", org_id.to_string());
        }
        req
    }

    fn build_post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.url, path);
        let mut req = self
            .http
            .post(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.config.api_key),
            )
            .header(reqwest::header::ACCEPT, "application/json");
        if let Some(org_id) = self.config.org_id {
            req = req.header("X-Grafana-Org-Id", org_id.to_string());
        }
        req
    }

    async fn fetch_health(&self) -> Result<RawHealth, String> {
        let resp = self
            .build_get("/api/health")
            .send()
            .await
            .map_err(|e| format!("Grafana /api/health unreachable: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("Grafana /api/health returned HTTP {status}"));
        }
        resp.json::<RawHealth>()
            .await
            .map_err(|e| format!("Invalid /api/health JSON: {e}"))
    }

    async fn fetch_org(&self) -> Result<RawOrg, AuthOrError> {
        let resp = self
            .build_get("/api/org")
            .send()
            .await
            .map_err(|e| AuthOrError::Transport(format!("Grafana /api/org request failed: {e}")))?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(AuthOrError::AuthRejected(status.as_u16()));
        }
        if !status.is_success() {
            return Err(AuthOrError::Transport(format!(
                "Grafana /api/org returned HTTP {status}"
            )));
        }
        resp.json::<RawOrg>()
            .await
            .map_err(|e| AuthOrError::Transport(format!("Invalid /api/org JSON: {e}")))
    }

    async fn fetch_datasources(&self) -> Result<Vec<RawDatasource>, String> {
        let resp = self
            .build_get("/api/datasources")
            .send()
            .await
            .map_err(|e| format!("Grafana /api/datasources request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("Grafana /api/datasources returned HTTP {status}"));
        }
        resp.json::<Vec<RawDatasource>>()
            .await
            .map_err(|e| format!("Invalid /api/datasources JSON: {e}"))
    }

    async fn fetch_dashboards(&self) -> Result<Vec<GrafanaDashboardSummary>, String> {
        let mut path = "/api/search?type=dash-db&limit=20".to_string();
        if let Some(tag) = &self.config.dashboard_tag {
            path.push_str("&tag=");
            path.push_str(&encode_query_component(tag));
        }

        let resp = self
            .build_get(&path)
            .send()
            .await
            .map_err(|e| format!("Grafana /api/search failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("Grafana /api/search returned HTTP {status}"));
        }
        let items = resp
            .json::<Vec<RawDashboard>>()
            .await
            .map_err(|e| format!("Invalid /api/search JSON: {e}"))?;

        let base_link = self.config.link_base_url();
        Ok(items
            .into_iter()
            .map(|d| {
                let url = base_link.and_then(|base| {
                    d.url.as_deref().map(|rel| {
                        if rel.starts_with("http://") || rel.starts_with("https://") {
                            rel.to_string()
                        } else if rel.starts_with('/') {
                            format!("{base}{rel}")
                        } else {
                            format!("{base}/{rel}")
                        }
                    })
                });
                GrafanaDashboardSummary {
                    uid: d.uid.unwrap_or_default(),
                    title: d.title,
                    folder_title: d.folder_title,
                    tags: d.tags,
                    url,
                }
            })
            .collect())
    }

    async fn fetch_alerts(&self) -> Result<Vec<GrafanaAlert>, String> {
        let resp = self
            .build_get("/api/alertmanager/grafana/api/v2/alerts")
            .send()
            .await
            .map_err(|e| format!("Alerts endpoint unreachable: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!(
                "Alertmanager endpoint returned HTTP {status} (alerts optional)"
            ));
        }
        let raw = resp
            .json::<Vec<RawAlert>>()
            .await
            .map_err(|e| format!("Invalid alerts JSON: {e}"))?;

        Ok(raw
            .into_iter()
            .map(|a| {
                let name = a
                    .labels
                    .get("alertname")
                    .cloned()
                    .unwrap_or_else(|| "Unnamed Alert".to_string());
                let severity = a
                    .labels
                    .get("severity")
                    .cloned()
                    .unwrap_or_else(|| "info".to_string());
                let state = a
                    .status
                    .and_then(|s| s.state)
                    .unwrap_or_else(|| "active".to_string());
                let summary = a
                    .annotations
                    .get("summary")
                    .or_else(|| a.annotations.get("description"))
                    .or_else(|| a.annotations.get("message"))
                    .map(|s| scrub_ips(s));
                let mut labels: Vec<(String, String)> = a
                    .labels
                    .into_iter()
                    .filter(|(k, v)| {
                        !k.starts_with("__")
                            && !matches!(
                                k.as_str(),
                                "alertname"
                                    | "severity"
                                    | "instance"
                                    | "host"
                                    | "node"
                                    | "pod"
                                    | "ip"
                                    | "address"
                                    | "id"
                            )
                            && !contains_ip_like(v)
                    })
                    .collect();
                labels.sort_by(|x, y| x.0.cmp(&y.0));
                labels.truncate(6);

                GrafanaAlert {
                    name,
                    state,
                    severity,
                    summary,
                    starts_at: a.starts_at,
                    labels,
                }
            })
            .collect())
    }

    async fn fetch_annotations(&self) -> Result<Vec<GrafanaAnnotation>, String> {
        let resp = self
            .build_get("/api/annotations?limit=12")
            .send()
            .await
            .map_err(|e| format!("Annotations request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("Annotations endpoint returned HTTP {status}"));
        }
        let raw = resp
            .json::<Vec<RawAnnotation>>()
            .await
            .map_err(|e| format!("Invalid annotations JSON: {e}"))?;
        Ok(raw
            .into_iter()
            .map(|a| {
                let text = match a.alert_name.as_deref().map(str::trim) {
                    Some(name) if !name.is_empty() => {
                        match (
                            a.prev_state.as_deref().map(str::trim).filter(|s| !s.is_empty()),
                            a.new_state.as_deref().map(str::trim).filter(|s| !s.is_empty()),
                        ) {
                            (Some(prev), Some(new)) => format!("{name} ({prev} → {new})"),
                            (None, Some(new)) => format!("{name} ({new})"),
                            _ => name.to_string(),
                        }
                    }
                    _ => scrub_ips(&a.text),
                };

                let tags = a
                    .tags
                    .into_iter()
                    .filter(|t| {
                        !t.starts_with("instance:")
                            && !t.starts_with("alertname:")
                            && !t.starts_with("id:")
                            && !contains_ip_like(t)
                    })
                    .collect();

                GrafanaAnnotation {
                    id: a.id,
                    text,
                    tags,
                    time_ms: a.time,
                }
            })
            .collect())
    }

    async fn query_datasource(
        &self,
        ds: &GrafanaDatasourceSummary,
    ) -> Result<DsQueryResponse, String> {
        let queries = self.config.resolved_queries(&ds.ds_type);
        let is_influx = ds.ds_type.to_ascii_lowercase().contains("influx");
        let now_ms = Utc::now().timestamp_millis();
        let from_ms = now_ms - self.config.range_duration_ms();
        let interval_ms = self.config.interval_ms();

        let mut query_specs = vec![
            build_query_item("cpu", &queries.cpu, ds, is_influx, interval_ms, false),
            build_query_item("memory", &queries.memory, ds, is_influx, interval_ms, false),
            build_query_item("disk", &queries.disk, ds, is_influx, interval_ms, false),
            build_query_item("load", &queries.load, ds, is_influx, interval_ms, false),
            build_query_item("network", &queries.network, ds, is_influx, interval_ms, false),
            build_query_item("uptime", &queries.uptime, ds, is_influx, interval_ms, true),
        ];

        if !is_influx
            && let Some(guests_expr) = queries.guests.as_deref()
            && !guests_expr.trim().is_empty()
        {
            query_specs.push(build_query_item(
                "guests",
                guests_expr,
                ds,
                false,
                interval_ms,
                false,
            ));
        }

        if let Some(svc_expr) = queries.services.as_deref()
            && !svc_expr.trim().is_empty()
        {
            query_specs.push(build_query_item(
                "services",
                svc_expr,
                ds,
                is_influx,
                interval_ms,
                true,
            ));
        }

        let payload = serde_json::json!({
            "from": from_ms.to_string(),
            "to": now_ms.to_string(),
            "queries": query_specs,
        });

        let mut parsed = self.send_ds_query(payload).await?;

        // If default Prometheus `probe_success` returned no frames, fall back cleanly to `up`
        let need_up_fallback = !is_influx
            && queries.services_is_default
            && parsed
                .results
                .get("services")
                .is_none_or(|r| r.frames.is_empty() && r.error.is_none());

        if need_up_fallback {
            let fallback_payload = serde_json::json!({
                "from": from_ms.to_string(),
                "to": now_ms.to_string(),
                "queries": [build_query_item("services", "up", ds, false, interval_ms, true)],
            });
            if let Ok(fallback) = self.send_ds_query(fallback_payload).await
                && let Some(svc_res) = fallback.results.get("services").cloned()
                && !svc_res.frames.is_empty()
            {
                parsed.results.insert("services".to_string(), svc_res);
            }
        }

        Ok(parsed)
    }

    async fn send_ds_query(&self, payload: serde_json::Value) -> Result<DsQueryResponse, String> {
        let resp = self
            .build_post("/api/ds/query")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Grafana /api/ds/query request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() && status != reqwest::StatusCode::MULTI_STATUS {
            let body_snippet = resp.text().await.unwrap_or_default();
            let snippet = body_snippet.chars().take(180).collect::<String>();
            return Err(format!(
                "Grafana /api/ds/query returned HTTP {status}: {snippet}"
            ));
        }
        resp.json::<DsQueryResponse>()
            .await
            .map_err(|e| format!("Invalid /api/ds/query JSON: {e}"))
    }

    pub async fn sync(&self) -> TelemetrySnapshot {
        let mut diagnostics = Vec::new();

        let (health_res, org_res, ds_res, dash_res, alert_res, ann_res) = tokio::join!(
            self.fetch_health(),
            self.fetch_org(),
            self.fetch_datasources(),
            self.fetch_dashboards(),
            self.fetch_alerts(),
            self.fetch_annotations(),
        );

        let now_str = Utc::now().to_rfc3339();

        let org = match org_res {
            Ok(org) => org,
            Err(AuthOrError::AuthRejected(code)) => {
                let msg = format!(
                    "Grafana rejected the API key (HTTP {code}). Verify GRAFANA_API_KEY and Service Account Viewer permissions."
                );
                return TelemetrySnapshot {
                    state: TelemetryState::Error,
                    configured: true,
                    grafana_version: health_res.as_ref().ok().and_then(|h| h.version.clone()),
                    grafana_database: health_res.as_ref().ok().and_then(|h| h.database.clone()),
                    org_name: None,
                    active_datasource: None,
                    datasources: Vec::new(),
                    dashboards: Vec::new(),
                    alerts: Vec::new(),
                    annotations: Vec::new(),
                    metrics: ClusterMetrics::default(),
                    service_probes: default_unmonitored_probes(),
                    time_range: self.config.time_range.clone(),
                    poll_interval_secs: self.config.poll_interval_secs,
                    last_synced_at: Some(now_str),
                    last_error: Some(msg.clone()),
                    diagnostics: vec![msg],
                };
            }
            Err(AuthOrError::Transport(err)) => {
                let err_msg = match health_res {
                    Err(ref h_err) => h_err.clone(),
                    Ok(_) => err,
                };
                return TelemetrySnapshot {
                    state: TelemetryState::Error,
                    configured: true,
                    grafana_version: health_res.as_ref().ok().and_then(|h| h.version.clone()),
                    grafana_database: health_res.as_ref().ok().and_then(|h| h.database.clone()),
                    org_name: None,
                    active_datasource: None,
                    datasources: Vec::new(),
                    dashboards: Vec::new(),
                    alerts: Vec::new(),
                    annotations: Vec::new(),
                    metrics: ClusterMetrics::default(),
                    service_probes: default_unmonitored_probes(),
                    time_range: self.config.time_range.clone(),
                    poll_interval_secs: self.config.poll_interval_secs,
                    last_synced_at: Some(now_str),
                    last_error: Some(err_msg.clone()),
                    diagnostics: vec![err_msg],
                };
            }
        };

        let mut partial_issue = false;

        let health = match health_res {
            Ok(h) => Some(h),
            Err(err) => {
                partial_issue = true;
                diagnostics.push(err);
                None
            }
        };

        let raw_datasources = match ds_res {
            Ok(ds) => ds,
            Err(err) => {
                partial_issue = true;
                diagnostics.push(err);
                Vec::new()
            }
        };

        let datasources: Vec<GrafanaDatasourceSummary> = raw_datasources
            .iter()
            .map(|d| GrafanaDatasourceSummary {
                uid: d.uid.clone(),
                name: d.name.clone(),
                ds_type: d.ds_type.clone(),
                is_default: d.is_default,
            })
            .collect();

        let active_datasource = select_datasource(&datasources, &self.config, &mut diagnostics);

        let dashboards = match dash_res {
            Ok(d) => d,
            Err(err) => {
                partial_issue = true;
                diagnostics.push(err);
                Vec::new()
            }
        };

        let alerts = match alert_res {
            Ok(a) => a,
            Err(err) => {
                diagnostics.push(err);
                Vec::new()
            }
        };

        let annotations = match ann_res {
            Ok(a) => a,
            Err(err) => {
                diagnostics.push(err);
                Vec::new()
            }
        };

        let mut metrics = ClusterMetrics::default();
        let mut service_probes = default_unmonitored_probes();
        let mut last_error: Option<String> = None;

        if let Some(ref ds) = active_datasource {
            match self.query_datasource(ds).await {
                Ok(resp) => {
                    metrics.cpu = parse_metric_series(
                        resp.results.get("cpu"),
                        "cpu",
                        "CPU Utilization",
                        "%",
                    );
                    metrics.memory = parse_metric_series(
                        resp.results.get("memory"),
                        "memory",
                        "Memory Utilization",
                        "%",
                    );
                    metrics.disk = parse_metric_series(
                        resp.results.get("disk"),
                        "disk",
                        "Disk Utilization",
                        "%",
                    );
                    let guests_series = parse_metric_series(
                        resp.results.get("guests"),
                        "load",
                        "Active PVE Guests",
                        "guests",
                    );
                    let node_load_series = parse_metric_series(
                        resp.results.get("load"),
                        "load",
                        "Load Average (1m)",
                        "",
                    );
                    metrics.load = if !guests_series.points.is_empty()
                        || (node_load_series.points.is_empty()
                            && guests_series.query_error.is_some())
                    {
                        guests_series
                    } else {
                        node_load_series
                    };
                    metrics.network = parse_metric_series(
                        resp.results.get("network"),
                        "network",
                        "Network Throughput",
                        "MB/s",
                    );

                    let uptime_series = parse_metric_series(
                        resp.results.get("uptime"),
                        "uptime",
                        "Uptime",
                        "s",
                    );
                    metrics.uptime_seconds = uptime_series.current;

                    for series in [
                        &metrics.cpu,
                        &metrics.memory,
                        &metrics.disk,
                        &metrics.load,
                        &metrics.network,
                        &uptime_series,
                    ] {
                        if let Some(err) = &series.query_error {
                            partial_issue = true;
                            diagnostics.push(format!("Query '{}' error: {err}", series.key));
                        }
                    }

                    if let Some(svc_res) = resp.results.get("services") {
                        if let Some(err) = &svc_res.error {
                            partial_issue = true;
                            diagnostics.push(format!("Service probe query error: {err}"));
                        } else {
                            service_probes = extract_service_probes(svc_res);
                        }
                    }
                }
                Err(err) => {
                    partial_issue = true;
                    last_error = Some(err.clone());
                    diagnostics.push(err);
                }
            }
        } else {
            partial_issue = true;
            diagnostics.push(
                "No Prometheus, VictoriaMetrics, or InfluxDB datasource found in Grafana."
                    .to_string(),
            );
        }

        let state = if partial_issue {
            TelemetryState::Degraded
        } else {
            TelemetryState::Connected
        };

        if diagnostics.is_empty() {
            diagnostics.push(format!(
                "Synchronized cleanly with Grafana ({}) via datasource '{}'.",
                org.name,
                active_datasource
                    .as_ref()
                    .map(|d| d.name.as_str())
                    .unwrap_or("none")
            ));
        }

        TelemetrySnapshot {
            state,
            configured: true,
            grafana_version: health.as_ref().and_then(|h| h.version.clone()),
            grafana_database: health.as_ref().and_then(|h| h.database.clone()),
            org_name: Some(org.name),
            active_datasource,
            datasources,
            dashboards,
            alerts,
            annotations,
            metrics,
            service_probes,
            time_range: self.config.time_range.clone(),
            poll_interval_secs: self.config.poll_interval_secs,
            last_synced_at: Some(now_str),
            last_error,
            diagnostics,
        }
    }
}

pub struct TelemetryStore {
    config: Arc<AppConfig>,
    client: Option<GrafanaClient>,
    snapshot: RwLock<TelemetrySnapshot>,
    tx: broadcast::Sender<TelemetrySnapshot>,
}

impl TelemetryStore {
    pub fn new(config: Arc<AppConfig>) -> Arc<Self> {
        let mut init_snapshot = if config.grafana.is_some() {
            TelemetrySnapshot::pending(&config)
        } else {
            TelemetrySnapshot::unconfigured(&config)
        };

        let client = match config.grafana.as_ref() {
            Some(gcfg) => match GrafanaClient::new(gcfg.clone()) {
                Ok(c) => Some(c),
                Err(err) => {
                    init_snapshot.state = TelemetryState::Error;
                    init_snapshot.last_error = Some(err.clone());
                    init_snapshot.diagnostics = vec![err];
                    None
                }
            },
            None => None,
        };

        let (tx, _) = broadcast::channel(16);

        Arc::new(Self {
            config,
            client,
            snapshot: RwLock::new(init_snapshot),
            tx,
        })
    }

    pub async fn snapshot(&self) -> TelemetrySnapshot {
        self.snapshot.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TelemetrySnapshot> {
        self.tx.subscribe()
    }

    pub async fn poll_once(&self) -> TelemetrySnapshot {
        let snap = match &self.client {
            Some(client) => client.sync().await,
            None => {
                if self.config.grafana.is_some() {
                    self.snapshot.read().await.clone()
                } else {
                    TelemetrySnapshot::unconfigured(&self.config)
                }
            }
        };

        {
            let mut guard = self.snapshot.write().await;
            *guard = snap.clone();
        }
        let _ = self.tx.send(snap.clone());
        snap
    }

    pub fn spawn_poller(self: Arc<Self>) {
        if self.client.is_none() {
            info!("Grafana telemetry disabled (GRAFANA_URL / GRAFANA_API_KEY not configured)");
            return;
        }
        let interval_secs = self.config.grafana_poll_interval_secs;
        tokio::spawn(async move {
            info!(interval_secs, "Starting Grafana background telemetry poller");
            let initial = self.poll_once().await;
            info!(state = ?initial.state, "Initial Grafana telemetry sync completed");

            let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            // Consume immediate first tick since we just ran initial sync
            ticker.tick().await;

            loop {
                ticker.tick().await;
                let snap = self.poll_once().await;
                match snap.state {
                    TelemetryState::Connected => {
                        debug!("Grafana telemetry refreshed cleanly");
                    }
                    TelemetryState::Degraded | TelemetryState::Error => {
                        warn!(state = ?snap.state, error = ?snap.last_error, "Grafana telemetry sync reported issues");
                    }
                    TelemetryState::Unconfigured => {}
                }
            }
        });
    }
}

fn is_supported_ds_type(ds_type: &str) -> bool {
    let lower = ds_type.to_ascii_lowercase();
    lower.contains("prometheus") || lower.contains("victoria") || lower.contains("influx")
}

fn select_datasource(
    datasources: &[GrafanaDatasourceSummary],
    config: &GrafanaConfig,
    diagnostics: &mut Vec<String>,
) -> Option<GrafanaDatasourceSummary> {
    if let Some(uid) = config.datasource_uid.as_deref() {
        if let Some(found) = datasources.iter().find(|d| d.uid == uid) {
            return Some(found.clone());
        }
        diagnostics.push(format!(
            "Configured GRAFANA_DATASOURCE_UID '{uid}' was not found; falling back to auto-discovery."
        ));
    }

    if let Some(pref_type) = config.datasource_type.as_deref() {
        let match_fn = |d: &&GrafanaDatasourceSummary| {
            let lower = d.ds_type.to_ascii_lowercase();
            if pref_type == "influxdb" {
                lower.contains("influx")
            } else {
                lower.contains("prometheus") || lower.contains("victoria")
            }
        };
        if let Some(found) = datasources.iter().filter(match_fn).find(|d| d.is_default) {
            return Some(found.clone());
        }
        if let Some(found) = datasources.iter().find(match_fn) {
            return Some(found.clone());
        }
    }

    if let Some(found) = datasources
        .iter()
        .find(|d| d.is_default && is_supported_ds_type(&d.ds_type))
    {
        return Some(found.clone());
    }

    if let Some(found) = datasources.iter().find(|d| {
        let lower = d.ds_type.to_ascii_lowercase();
        lower.contains("prometheus") || lower.contains("victoria")
    }) {
        return Some(found.clone());
    }

    datasources
        .iter()
        .find(|d| is_supported_ds_type(&d.ds_type))
        .cloned()
}

fn build_query_item(
    ref_id: &str,
    expr: &str,
    ds: &GrafanaDatasourceSummary,
    is_influx: bool,
    interval_ms: u64,
    instant: bool,
) -> serde_json::Value {
    if is_influx {
        serde_json::json!({
            "refId": ref_id,
            "datasource": { "uid": ds.uid, "type": ds.ds_type },
            "rawQuery": true,
            "query": expr,
            "resultFormat": "time_series",
            "intervalMs": interval_ms,
            "maxDataPoints": MAX_CHART_POINTS,
        })
    } else {
        serde_json::json!({
            "refId": ref_id,
            "datasource": { "uid": ds.uid, "type": ds.ds_type },
            "expr": expr,
            "range": !instant,
            "instant": instant,
            "intervalMs": interval_ms,
            "maxDataPoints": MAX_CHART_POINTS,
            "format": "time_series",
        })
    }
}

pub fn parse_metric_series(
    result: Option<&DsQueryResult>,
    key: &str,
    label: &str,
    unit: &str,
) -> MetricSeries {
    let mut series = MetricSeries::empty(key, label, unit);
    let Some(res) = result else {
        return series;
    };

    if let Some(err) = &res.error
        && !err.trim().is_empty()
    {
        series.query_error = Some(err.clone());
        return series;
    }

    let Some(frame) = res.frames.first() else {
        return series;
    };

    let mut raw_points = extract_frame_points(frame);
    if raw_points.is_empty() {
        return series;
    }

    raw_points.sort_by_key(|p| p.ts_ms);
    let points = downsample_points(raw_points, MAX_CHART_POINTS);

    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    let mut sum = 0.0;
    for p in &points {
        if p.value < min_v {
            min_v = p.value;
        }
        if p.value > max_v {
            max_v = p.value;
        }
        sum += p.value;
    }

    series.current = points.last().map(|p| p.value);
    series.min = Some(min_v);
    series.max = Some(max_v);
    series.avg = Some(sum / points.len() as f64);
    series.points = points;
    series
}

fn extract_frame_points(frame: &DataFrame) -> Vec<TelemetryPoint> {
    let fields = &frame.schema.fields;
    let values = &frame.data.values;

    let time_idx = fields
        .iter()
        .position(|f| {
            f.field_type.as_deref() == Some("time")
                || f.name.eq_ignore_ascii_case("time")
                || f.name.eq_ignore_ascii_case("timestamp")
        })
        .unwrap_or(0);

    let val_idx = fields
        .iter()
        .enumerate()
        .find(|(idx, f)| {
            *idx != time_idx
                && (f.field_type.as_deref() == Some("number")
                    || f.field_type.is_none()
                    || f.name.eq_ignore_ascii_case("value")
                    || f.name.eq_ignore_ascii_case("mean")
                    || f.name.eq_ignore_ascii_case("last"))
        })
        .map(|(idx, _)| idx)
        .or(if values.len() > 1 { Some(1) } else { None });

    let Some(val_idx) = val_idx else {
        return Vec::new();
    };

    let Some(times) = values.get(time_idx) else {
        return Vec::new();
    };
    let Some(nums) = values.get(val_idx) else {
        return Vec::new();
    };

    let mut points = Vec::with_capacity(times.len().min(nums.len()));
    for (t_val, v_val) in times.iter().zip(nums.iter()) {
        let Some(ts_ms) = parse_timestamp_ms(t_val) else {
            continue;
        };
        let Some(val) = parse_finite_f64(v_val) else {
            continue;
        };
        points.push(TelemetryPoint { ts_ms, value: val });
    }
    points
}

fn parse_timestamp_ms(val: &serde_json::Value) -> Option<i64> {
    match val {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i.abs() < 100_000_000_000 {
                    Some(i * 1000)
                } else {
                    Some(i)
                }
            } else if let Some(f) = n.as_f64()
                && f.is_finite()
            {
                let i = f.round() as i64;
                if i.abs() < 100_000_000_000 {
                    Some(i * 1000)
                } else {
                    Some(i)
                }
            } else {
                None
            }
        }
        serde_json::Value::String(s) => {
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                Some(dt.timestamp_millis())
            } else if let Ok(n) = s.parse::<i64>() {
                if n.abs() < 100_000_000_000 {
                    Some(n * 1000)
                } else {
                    Some(n)
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_finite_f64(val: &serde_json::Value) -> Option<f64> {
    match val {
        serde_json::Value::Number(n) => n.as_f64().filter(|v| v.is_finite()),
        serde_json::Value::String(s) => s.parse::<f64>().ok().filter(|v| v.is_finite()),
        serde_json::Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn downsample_points(points: Vec<TelemetryPoint>, max_len: usize) -> Vec<TelemetryPoint> {
    if points.len() <= max_len || max_len < 2 {
        return points;
    }
    let last_idx = points.len() - 1;
    let mut sampled = Vec::with_capacity(max_len);
    for i in 0..max_len {
        let idx = i * last_idx / (max_len - 1);
        sampled.push(points[idx].clone());
    }
    sampled
}

fn contains_ip_like(raw: &str) -> bool {
    raw.split(|c: char| !c.is_ascii_digit() && c != '.')
        .any(|tok| tok.parse::<Ipv4Addr>().is_ok())
}

fn scrub_ips(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let cand = &raw[start..i];
            if cand.parse::<Ipv4Addr>().is_ok() {
                // Also consume optional :port immediately following the IPv4 address
                if i < bytes.len() && bytes[i] == b':' {
                    let port_start = i + 1;
                    let mut p = port_start;
                    while p < bytes.len() && bytes[p].is_ascii_digit() {
                        p += 1;
                    }
                    if p > port_start {
                        i = p;
                    }
                }
                out.push_str("[internal]");
                continue;
            }
            out.push_str(cand);
        } else {
            let ch = raw[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    out
}

fn extract_service_probes(svc_res: &DsQueryResult) -> Vec<ServiceProbeStatus> {
    let mut extracted: Vec<(Vec<String>, Option<String>, bool)> = Vec::new();

    for frame in &svc_res.frames {
        let pts = extract_frame_points(frame);
        let Some(last) = pts.last() else {
            continue;
        };
        let is_up = last.value >= 0.5;

        let mut candidates = Vec::new();
        if let Some(name) = &frame.schema.name
            && !name.trim().is_empty()
            && !contains_ip_like(name)
        {
            candidates.push(name.clone());
        }
        let mut preferred_target: Option<String> = None;

        for field in &frame.schema.fields {
            if let Some(labels) = &field.labels {
                for key in ["name", "service", "container", "target", "job"] {
                    if let Some(val) = labels.get(key)
                        && !val.trim().is_empty()
                        && !contains_ip_like(val)
                    {
                        if preferred_target.is_none() && key != "job" {
                            preferred_target = Some(val.clone());
                        }
                        candidates.push(val.clone());
                    }
                }
                if preferred_target.is_none()
                    && let Some(job_val) = labels.get("job")
                    && !job_val.trim().is_empty()
                    && !contains_ip_like(job_val)
                {
                    preferred_target = Some(job_val.clone());
                }
            }
        }

        if preferred_target.is_none()
            && let Some(name) = &frame.schema.name
            && !name.trim().is_empty()
            && !contains_ip_like(name)
        {
            preferred_target = Some(name.clone());
        }

        if !candidates.is_empty() {
            extracted.push((candidates, preferred_target, is_up));
        }
    }

    let mut probes = Vec::with_capacity(HOMELAB_SERVICES.len());

    for svc in HOMELAB_SERVICES {
        let matched = extracted.iter().find(|(cands, _, _)| {
            cands.iter().any(|cand| {
                let lower = cand.to_ascii_lowercase();
                svc.probe_keywords.iter().any(|kw| lower.contains(kw))
            })
        });

        if let Some((_, target, up)) = matched {
            probes.push(ServiceProbeStatus {
                service_id: svc.id.to_string(),
                name: svc.name.to_string(),
                up: Some(*up),
                target: target.clone(),
                source: "grafana".to_string(),
            });
        } else {
            probes.push(ServiceProbeStatus {
                service_id: svc.id.to_string(),
                name: svc.name.to_string(),
                up: None,
                target: None,
                source: "unmatched".to_string(),
            });
        }
    }

    probes
}

fn encode_query_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

enum AuthOrError {
    AuthRejected(u16),
    Transport(String),
}

#[derive(Debug, Deserialize)]
struct RawHealth {
    database: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawOrg {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawDatasource {
    uid: String,
    name: String,
    #[serde(rename = "type")]
    ds_type: String,
    #[serde(rename = "isDefault", default)]
    is_default: bool,
}

#[derive(Debug, Deserialize)]
struct RawDashboard {
    uid: Option<String>,
    title: String,
    #[serde(rename = "folderTitle")]
    folder_title: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawAlert {
    #[serde(default)]
    labels: HashMap<String, String>,
    #[serde(default)]
    annotations: HashMap<String, String>,
    #[serde(rename = "startsAt")]
    starts_at: Option<String>,
    status: Option<RawAlertStatus>,
}

#[derive(Debug, Deserialize)]
struct RawAlertStatus {
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawAnnotation {
    id: i64,
    #[serde(default)]
    text: String,
    #[serde(default)]
    tags: Vec<String>,
    time: i64,
    #[serde(rename = "alertName")]
    alert_name: Option<String>,
    #[serde(rename = "prevState")]
    prev_state: Option<String>,
    #[serde(rename = "newState")]
    new_state: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DsQueryResponse {
    #[serde(default)]
    pub results: HashMap<String, DsQueryResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DsQueryResult {
    pub error: Option<String>,
    #[serde(default)]
    pub frames: Vec<DataFrame>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataFrame {
    pub schema: DataFrameSchema,
    pub data: DataFrameData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataFrameSchema {
    pub name: Option<String>,
    #[serde(default)]
    pub fields: Vec<DataFrameField>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataFrameField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataFrameData {
    #[serde(default)]
    pub values: Vec<Vec<serde_json::Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GrafanaQueryOverrides;
    use axum::{
        Json, Router,
        extract::State,
        http::{HeaderMap, StatusCode},
        routing::{get, post},
    };
    use tokio::net::TcpListener;

    #[test]
    fn parses_prometheus_and_influx_dataframes_with_nulls_and_errors() {
        let prom_json = serde_json::json!({
            "results": {
                "cpu": {
                    "frames": [{
                        "schema": {
                            "fields": [
                                { "name": "Time", "type": "time" },
                                { "name": "Value", "type": "number" }
                            ]
                        },
                        "data": {
                            "values": [
                                [1700000000000_i64, 1700000300000_i64, 1700000600000_i64],
                                [12.5, null, 27.5]
                            ]
                        }
                    }]
                },
                "disk": {
                    "error": "query timeout"
                }
            }
        });

        let parsed: DsQueryResponse = serde_json::from_value(prom_json).unwrap();
        let cpu = parse_metric_series(parsed.results.get("cpu"), "cpu", "CPU", "%");
        assert_eq!(cpu.points.len(), 2);
        assert_eq!(cpu.current, Some(27.5));
        assert_eq!(cpu.min, Some(12.5));
        assert_eq!(cpu.max, Some(27.5));
        assert_eq!(cpu.avg, Some(20.0));
        assert!(cpu.query_error.is_none());

        let disk = parse_metric_series(parsed.results.get("disk"), "disk", "Disk", "%");
        assert!(disk.points.is_empty());
        assert_eq!(disk.query_error.as_deref(), Some("query timeout"));

        let influx_json = serde_json::json!({
            "results": {
                "memory": {
                    "frames": [{
                        "schema": {
                            "fields": [
                                { "name": "Time", "type": "time" },
                                { "name": "mean", "type": "number" }
                            ]
                        },
                        "data": {
                            "values": [
                                ["2026-11-19T01:00:00Z", "2026-11-19T01:05:00Z"],
                                [44.0, 46.5]
                            ]
                        }
                    }]
                }
            }
        });
        let influx_parsed: DsQueryResponse = serde_json::from_value(influx_json).unwrap();
        let mem = parse_metric_series(influx_parsed.results.get("memory"), "memory", "Memory", "%");
        assert_eq!(mem.points.len(), 2);
        assert_eq!(mem.current, Some(46.5));
        assert!(mem.points[0].ts_ms > 1_700_000_000_000);
    }

    #[test]
    fn downsamples_large_series_preserving_endpoints() {
        let raw = (0..200)
            .map(|i| TelemetryPoint {
                ts_ms: 1700000000000 + i * 1000,
                value: i as f64,
            })
            .collect();
        let sampled = downsample_points(raw, 72);
        assert_eq!(sampled.len(), 72);
        assert_eq!(sampled.first().unwrap().value, 0.0);
        assert_eq!(sampled.last().unwrap().value, 199.0);
    }

    #[tokio::test]
    async fn end_to_end_mock_grafana_sync_connected_and_redacts_secret() {
        #[derive(Clone)]
        struct MockState {
            expected_token: String,
        }

        async fn check_auth(headers: &HeaderMap, expected: &str) -> Result<(), StatusCode> {
            let auth = headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok());
            if auth == Some(&format!("Bearer {expected}")) {
                Ok(())
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }

        let state = MockState {
            expected_token: "glsa_secret_mock_token_xyz".to_string(),
        };

        let mock_app = Router::new()
            .route(
                "/api/health",
                get(|| async {
                    Json(serde_json::json!({
                        "database": "ok",
                        "version": "11.3.0"
                    }))
                }),
            )
            .route(
                "/api/org",
                get(|State(st): State<MockState>, headers: HeaderMap| async move {
                    check_auth(&headers, &st.expected_token).await?;
                    Ok::<_, StatusCode>(Json(serde_json::json!({ "id": 1, "name": "Homelab Org" })))
                }),
            )
            .route(
                "/api/datasources",
                get(|State(st): State<MockState>, headers: HeaderMap| async move {
                    check_auth(&headers, &st.expected_token).await?;
                    Ok::<_, StatusCode>(Json(serde_json::json!([
                        {
                            "id": 1,
                            "uid": "prom-main",
                            "name": "Prometheus Homelab",
                            "type": "prometheus",
                            "isDefault": true,
                            "url": "http://10.0.0.5:9090"
                        }
                    ])))
                }),
            )
            .route(
                "/api/search",
                get(|State(st): State<MockState>, headers: HeaderMap| async move {
                    check_auth(&headers, &st.expected_token).await?;
                    Ok::<_, StatusCode>(Json(serde_json::json!([
                        {
                            "uid": "pve-overview",
                            "title": "Proxmox VE Cluster",
                            "folderTitle": "Infrastructure",
                            "tags": ["proxmox", "homelab"],
                            "url": "/d/pve-overview/proxmox-ve-cluster"
                        }
                    ])))
                }),
            )
            .route(
                "/api/alertmanager/grafana/api/v2/alerts",
                get(|State(st): State<MockState>, headers: HeaderMap| async move {
                    check_auth(&headers, &st.expected_token).await?;
                    Ok::<_, StatusCode>(Json(serde_json::json!([
                        {
                            "labels": {
                                "alertname": "ZfsPoolScrubWarning",
                                "severity": "warning",
                                "host": "pve-node-01"
                            },
                            "annotations": {
                                "summary": "ZFS scrub running on tank"
                            },
                            "startsAt": "2026-11-19T01:00:00Z",
                            "status": { "state": "active" }
                        }
                    ])))
                }),
            )
            .route(
                "/api/annotations",
                get(|State(st): State<MockState>, headers: HeaderMap| async move {
                    check_auth(&headers, &st.expected_token).await?;
                    Ok::<_, StatusCode>(Json(serde_json::json!([
                        {
                            "id": 42,
                            "text": "Deployed urashyl.dev v2",
                            "tags": ["deploy"],
                            "time": 1700000600000_i64
                        }
                    ])))
                }),
            )
            .route(
                "/api/ds/query",
                post(|State(st): State<MockState>, headers: HeaderMap| async move {
                    check_auth(&headers, &st.expected_token).await?;
                    Ok::<_, StatusCode>(Json(serde_json::json!({
                        "results": {
                            "cpu": {
                                "frames": [{
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                    "data": { "values": [[1700000000000_i64, 1700000300000_i64], [14.2, 18.7]] }
                                }]
                            },
                            "memory": {
                                "frames": [{
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                    "data": { "values": [[1700000000000_i64, 1700000300000_i64], [52.0, 54.5]] }
                                }]
                            },
                            "disk": {
                                "frames": [{
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                    "data": { "values": [[1700000000000_i64, 1700000300000_i64], [41.0, 41.3]] }
                                }]
                            },
                            "load": {
                                "frames": [{
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                    "data": { "values": [[1700000000000_i64, 1700000300000_i64], [0.42, 0.58]] }
                                }]
                            },
                            "network": {
                                "frames": [{
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                    "data": { "values": [[1700000000000_i64, 1700000300000_i64], [2.4, 3.1]] }
                                }]
                            },
                            "uptime": {
                                "frames": [{
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                    "data": { "values": [[1700000300000_i64], [3_628_800.0]] }
                                }]
                            },
                            "services": {
                                "frames": [
                                    {
                                        "schema": {
                                            "fields": [
                                                { "name": "Time", "type": "time" },
                                                { "name": "Value", "type": "number", "labels": { "instance": "https://vault.urashyl.dev", "service": "bitwarden" } }
                                            ]
                                        },
                                        "data": { "values": [[1700000300000_i64], [1.0]] }
                                    },
                                    {
                                        "schema": {
                                            "fields": [
                                                { "name": "Time", "type": "time" },
                                                { "name": "Value", "type": "number", "labels": { "instance": "https://photos.urashyl.dev", "service": "immich" } }
                                            ]
                                        },
                                        "data": { "values": [[1700000300000_i64], [0.0]] }
                                    }
                                ]
                            }
                        }
                    })))
                }),
            )
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, mock_app).await;
        });

        let gcfg = GrafanaConfig {
            url: format!("http://{addr}"),
            api_key: "glsa_secret_mock_token_xyz".to_string(),
            public_url: None,
            expose_links: false,
            org_id: None,
            datasource_uid: None,
            datasource_type: None,
            dashboard_tag: None,
            poll_interval_secs: 30,
            timeout_secs: 5,
            time_range: "6h".to_string(),
            insecure_tls: false,
            queries: GrafanaQueryOverrides::default(),
        };

        let app_cfg = Arc::new(AppConfig {
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            grafana: Some(gcfg),
            grafana_time_range: "6h".to_string(),
            grafana_poll_interval_secs: 30,
            grafana_timeout_secs: 5,
        });

        let store = TelemetryStore::new(app_cfg);
        let snap = store.poll_once().await;

        assert_eq!(snap.state, TelemetryState::Connected);
        assert_eq!(snap.grafana_version.as_deref(), Some("11.3.0"));
        assert_eq!(snap.org_name.as_deref(), Some("Homelab Org"));
        assert_eq!(snap.metrics.cpu.current, Some(18.7));
        assert_eq!(snap.metrics.memory.current, Some(54.5));
        assert_eq!(snap.metrics.uptime_seconds, Some(3_628_800.0));
        assert_eq!(snap.dashboards.len(), 1);
        assert!(snap.dashboards[0].url.is_none(), "private URL links should remain hidden unless configured");
        assert_eq!(snap.alerts.len(), 1);
        assert_eq!(snap.alerts[0].name, "ZfsPoolScrubWarning");
        assert_eq!(snap.annotations.len(), 1);
        assert_eq!(snap.probe_for_service("bitwarden").and_then(|p| p.up), Some(true));
        assert_eq!(snap.probe_for_service("immich").and_then(|p| p.up), Some(false));

        let serialized = serde_json::to_string(&snap).unwrap();
        assert!(!serialized.contains("glsa_secret_mock_token_xyz"));
        assert!(!serialized.contains("10.0.0.5:9090"));
    }

    #[tokio::test]
    async fn unauthorized_token_reports_error_and_missing_alertmanager_still_connects() {
        let mock_401 = Router::new()
            .route("/api/health", get(|| async { Json(serde_json::json!({ "version": "11.0.0", "database": "ok" })) }))
            .route("/api/org", get(|| async { StatusCode::UNAUTHORIZED }));

        let l1 = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let a1 = l1.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(l1, mock_401).await;
        });

        let client_401 = GrafanaClient::new(GrafanaConfig {
            url: format!("http://{a1}"),
            api_key: "bad-key".to_string(),
            public_url: None,
            expose_links: false,
            org_id: None,
            datasource_uid: None,
            datasource_type: None,
            dashboard_tag: None,
            poll_interval_secs: 30,
            timeout_secs: 5,
            time_range: "6h".to_string(),
            insecure_tls: false,
            queries: GrafanaQueryOverrides::default(),
        })
        .unwrap();

        let err_snap = client_401.sync().await;
        assert_eq!(err_snap.state, TelemetryState::Error);
        assert!(err_snap.last_error.as_deref().unwrap_or("").contains("HTTP 401"));

        // Mock where alertmanager returns 404 (disabled/unavailable), but everything else works
        let mock_no_alerts = Router::new()
            .route("/api/health", get(|| async { Json(serde_json::json!({ "version": "11.1.0", "database": "ok" })) }))
            .route("/api/org", get(|| async { Json(serde_json::json!({ "id": 1, "name": "Org" })) }))
            .route("/api/datasources", get(|| async {
                Json(serde_json::json!([{ "uid": "prom", "name": "Prom", "type": "prometheus", "isDefault": true }]))
            }))
            .route("/api/search", get(|| async { Json(serde_json::json!([])) }))
            .route("/api/alertmanager/grafana/api/v2/alerts", get(|| async { StatusCode::NOT_FOUND }))
            .route("/api/annotations", get(|| async { Json(serde_json::json!([])) }))
            .route("/api/ds/query", post(|| async { Json(serde_json::json!({ "results": {} })) }));

        let l2 = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let a2 = l2.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(l2, mock_no_alerts).await;
        });

        let client_ok = GrafanaClient::new(GrafanaConfig {
            url: format!("http://{a2}"),
            api_key: "good-key".to_string(),
            public_url: None,
            expose_links: false,
            org_id: None,
            datasource_uid: None,
            datasource_type: None,
            dashboard_tag: None,
            poll_interval_secs: 30,
            timeout_secs: 5,
            time_range: "6h".to_string(),
            insecure_tls: false,
            queries: GrafanaQueryOverrides::default(),
        })
        .unwrap();

        let ok_snap = client_ok.sync().await;
        assert_eq!(ok_snap.state, TelemetryState::Connected);
        assert!(ok_snap.alerts.is_empty());
    }

    #[tokio::test]
    async fn pve_exporter_sync_maps_guests_and_scrubs_internal_ips_and_unlisted_vms() {
        let pve_mock = Router::new()
            .route("/api/health", get(|| async { Json(serde_json::json!({ "version": "13.2.3", "database": "ok" })) }))
            .route("/api/org", get(|| async { Json(serde_json::json!({ "id": 1, "name": "Main Org." })) }))
            .route("/api/datasources", get(|| async {
                Json(serde_json::json!([{ "uid": "prom-pve", "name": "prometheus", "type": "prometheus", "isDefault": true }]))
            }))
            .route("/api/search", get(|| async { Json(serde_json::json!([])) }))
            .route("/api/alertmanager/grafana/api/v2/alerts", get(|| async {
                Json(serde_json::json!([
                    {
                        "labels": {
                            "alertname": "HighDisk",
                            "severity": "warning",
                            "instance": "192.168.100.1:8006",
                            "id": "lxc/103",
                            "job": "pve"
                        },
                        "annotations": {
                            "summary": "High disk on 192.168.100.1:8006 (85.2%)"
                        },
                        "startsAt": "2026-11-19T01:00:00Z",
                        "status": { "state": "active" }
                    }
                ]))
            }))
            .route("/api/annotations", get(|| async {
                Json(serde_json::json!([
                    {
                        "id": 309,
                        "alertName": "High CPU Spike (> 85%)",
                        "prevState": "Pending",
                        "newState": "Normal",
                        "text": "High CPU Spike {id=lxc/125, instance=192.168.100.1:8006}",
                        "tags": [
                            "alertname:High CPU Spike (> 85%)",
                            "instance:192.168.100.1:8006",
                            "grafana_folder:Infrastructure"
                        ],
                        "time": 1795056480000_i64
                    },
                    {
                        "id": 310,
                        "text": "Checked node 192.168.100.1:8006 after kernel 6.8.12 upgrade",
                        "tags": ["maintenance", "node:192.168.100.1"],
                        "time": 1795056500000_i64
                    }
                ]))
            }))
            .route("/api/ds/query", post(|| async {
                Json(serde_json::json!({
                    "results": {
                        "cpu": {
                            "frames": [{
                                "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                "data": { "values": [[1795056000000_i64, 1795056300000_i64], [5.8, 3.1]] }
                            }]
                        },
                        "guests": {
                            "frames": [{
                                "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number" }] },
                                "data": { "values": [[1795056000000_i64, 1795056300000_i64], [21.0, 22.0]] }
                            }]
                        },
                        "services": {
                            "frames": [
                                {
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number", "labels": { "id": "lxc/100", "job": "pve", "name": "immich", "instance": "192.168.100.1:8006" } }] },
                                    "data": { "values": [[1795056300000_i64], [1.0]] }
                                },
                                {
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number", "labels": { "id": "lxc/101", "job": "pve", "name": "jellyfin" } }] },
                                    "data": { "values": [[1795056300000_i64], [1.0]] }
                                },
                                {
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number", "labels": { "id": "lxc/105", "job": "pve", "name": "alpine-nextcloud" } }] },
                                    "data": { "values": [[1795056300000_i64], [1.0]] }
                                },
                                {
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number", "labels": { "id": "lxc/118", "job": "pve", "name": "vaultwarden" } }] },
                                    "data": { "values": [[1795056300000_i64], [1.0]] }
                                },
                                {
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number", "labels": { "id": "lxc/119", "job": "pve", "name": "infisical" } }] },
                                    "data": { "values": [[1795056300000_i64], [1.0]] }
                                },
                                {
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number", "labels": { "id": "lxc/121", "job": "pve", "name": "memos" } }] },
                                    "data": { "values": [[1795056300000_i64], [1.0]] }
                                },
                                {
                                    "schema": { "fields": [{ "name": "Time", "type": "time" }, { "name": "Value", "type": "number", "labels": { "id": "lxc/112", "job": "pve", "name": "vpn-box" } }] },
                                    "data": { "values": [[1795056300000_i64], [1.0]] }
                                }
                            ]
                        }
                    }
                }))
            }));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, pve_mock).await;
        });

        let client = GrafanaClient::new(GrafanaConfig {
            url: format!("http://{addr}"),
            api_key: "glsa_pve_test".to_string(),
            public_url: None,
            expose_links: false,
            org_id: None,
            datasource_uid: None,
            datasource_type: None,
            dashboard_tag: None,
            poll_interval_secs: 30,
            timeout_secs: 5,
            time_range: "6h".to_string(),
            insecure_tls: false,
            queries: GrafanaQueryOverrides::default(),
        })
        .unwrap();

        let snap = client.sync().await;
        assert_eq!(snap.state, TelemetryState::Connected);
        assert_eq!(snap.metrics.load.label, "Active PVE Guests");
        assert_eq!(snap.metrics.load.unit, "guests");
        assert_eq!(snap.metrics.load.current, Some(22.0));

        assert_eq!(snap.service_probes.len(), 6);
        for svc_id in ["infisical", "bitwarden", "immich", "nextcloud", "jellyfin", "memos"] {
            assert_eq!(snap.probe_for_service(svc_id).and_then(|p| p.up), Some(true));
        }

        assert_eq!(snap.alerts.len(), 1);
        assert_eq!(snap.alerts[0].summary.as_deref(), Some("High disk on [internal] (85.2%)"));
        assert!(snap.alerts[0].labels.iter().all(|(k, _)| k != "instance" && k != "id"));

        assert_eq!(snap.annotations.len(), 2);
        assert_eq!(snap.annotations[0].text, "High CPU Spike (> 85%) (Pending → Normal)");
        assert_eq!(snap.annotations[1].text, "Checked node [internal] after kernel 6.8.12 upgrade");

        let serialized = serde_json::to_string(&snap).unwrap();
        assert!(!serialized.contains("192.168."));
        assert!(!serialized.contains("vpn-box"));
        assert!(!serialized.contains("lxc/103"));
        assert!(!serialized.contains("lxc/125"));
    }
}
