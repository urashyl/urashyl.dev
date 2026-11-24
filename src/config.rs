use std::{
    collections::HashMap,
    env, fmt, fs,
    net::SocketAddr,
    path::Path,
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub grafana: Option<GrafanaConfig>,
    pub grafana_time_range: String,
    pub grafana_poll_interval_secs: u64,
    pub grafana_timeout_secs: u64,
}

#[derive(Clone)]
pub struct GrafanaConfig {
    pub url: String,
    pub api_key: String,
    pub public_url: Option<String>,
    pub expose_links: bool,
    pub org_id: Option<u64>,
    pub datasource_uid: Option<String>,
    pub datasource_type: Option<String>,
    pub dashboard_tag: Option<String>,
    pub poll_interval_secs: u64,
    pub timeout_secs: u64,
    pub time_range: String,
    pub insecure_tls: bool,
    pub queries: GrafanaQueryOverrides,
}

#[derive(Debug, Clone, Default)]
pub struct GrafanaQueryOverrides {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub disk: Option<String>,
    pub load: Option<String>,
    pub guests: Option<String>,
    pub uptime: Option<String>,
    pub network: Option<String>,
    pub services: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedQueries {
    pub cpu: String,
    pub memory: String,
    pub disk: String,
    pub load: String,
    pub load_is_custom: bool,
    pub guests: Option<String>,
    pub uptime: String,
    pub network: String,
    pub services: Option<String>,
    pub services_is_default: bool,
}

impl fmt::Debug for GrafanaConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GrafanaConfig")
            .field("url", &self.url)
            .field("api_key", &"<redacted>")
            .field("public_url", &self.public_url)
            .field("expose_links", &self.expose_links)
            .field("org_id", &self.org_id)
            .field("datasource_uid", &self.datasource_uid)
            .field("datasource_type", &self.datasource_type)
            .field("dashboard_tag", &self.dashboard_tag)
            .field("poll_interval_secs", &self.poll_interval_secs)
            .field("timeout_secs", &self.timeout_secs)
            .field("time_range", &self.time_range)
            .field("insecure_tls", &self.insecure_tls)
            .field("queries", &self.queries)
            .finish()
    }
}

impl GrafanaConfig {
    pub fn resolved_queries(&self, detected_ds_type: &str) -> ResolvedQueries {
        let ds_kind = self
            .datasource_type
            .as_deref()
            .unwrap_or(detected_ds_type)
            .to_ascii_lowercase();

        if ds_kind.contains("influx") {
            ResolvedQueries {
                cpu: self.queries.cpu.clone().unwrap_or_else(|| {
                    r#"SELECT mean("cpu") * 100 FROM "system" WHERE $timeFilter GROUP BY time($__interval) fill(null)"#.to_string()
                }),
                memory: self.queries.memory.clone().unwrap_or_else(|| {
                    r#"SELECT mean("memused") / mean("memtotal") * 100 FROM "system" WHERE $timeFilter GROUP BY time($__interval) fill(null)"#.to_string()
                }),
                disk: self.queries.disk.clone().unwrap_or_else(|| {
                    r#"SELECT mean("used") / mean("total") * 100 FROM "disk" WHERE $timeFilter GROUP BY time($__interval) fill(null)"#.to_string()
                }),
                load: self.queries.load.clone().unwrap_or_else(|| {
                    r#"SELECT mean("load1") FROM "system" WHERE $timeFilter GROUP BY time($__interval) fill(null)"#.to_string()
                }),
                load_is_custom: self.queries.load.is_some(),
                guests: self.queries.guests.clone(),
                uptime: self.queries.uptime.clone().unwrap_or_else(|| {
                    r#"SELECT last("uptime") FROM "system" WHERE $timeFilter"#.to_string()
                }),
                network: self.queries.network.clone().unwrap_or_else(|| {
                    r#"SELECT (non_negative_derivative(mean("netin"), 1s) + non_negative_derivative(mean("netout"), 1s)) / 1048576 FROM "system" WHERE $timeFilter GROUP BY time($__interval) fill(null)"#.to_string()
                }),
                services: self.queries.services.clone(),
                services_is_default: self.queries.services.is_none(),
            }
        } else {
            let services_is_default = self.queries.services.is_none();
            let load_is_custom = self.queries.load.is_some();
            let guests = self.queries.guests.clone().or_else(|| {
                if load_is_custom {
                    None
                } else {
                    Some(r#"count(pve_up{id=~"lxc/.*|qemu/.*"} == 1)"#.to_string())
                }
            });

            ResolvedQueries {
                cpu: self.queries.cpu.clone().unwrap_or_else(|| {
                    r#"(avg(pve_cpu_usage_ratio{id=~"node/.*"}) * 100) or (100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100))"#
                        .to_string()
                }),
                memory: self.queries.memory.clone().unwrap_or_else(|| {
                    r#"((sum(pve_memory_usage_bytes{id=~"node/.*"}) / sum(pve_memory_size_bytes{id=~"node/.*"})) * 100) or ((1 - (sum(node_memory_MemAvailable_bytes) / sum(node_memory_MemTotal_bytes))) * 100)"#
                        .to_string()
                }),
                disk: self.queries.disk.clone().unwrap_or_else(|| {
                    r#"((sum(pve_disk_usage_bytes{id=~"node/.*"}) / sum(pve_disk_size_bytes{id=~"node/.*"})) * 100) or ((1 - (sum(node_filesystem_avail_bytes{fstype!~"tmpfs|overlay|squashfs"}) / sum(node_filesystem_size_bytes{fstype!~"tmpfs|overlay|squashfs"}))) * 100)"#
                        .to_string()
                }),
                load: self
                    .queries
                    .load
                    .clone()
                    .unwrap_or_else(|| "avg(node_load1)".to_string()),
                load_is_custom,
                guests,
                uptime: self.queries.uptime.clone().unwrap_or_else(|| {
                    r#"max(pve_uptime_seconds{id=~"node/.*"}) or max(node_time_seconds - node_boot_time_seconds)"#
                        .to_string()
                }),
                network: self.queries.network.clone().unwrap_or_else(|| {
                    r#"(sum(rate(pve_network_receive_bytes_total[5m]) + rate(pve_network_transmit_bytes_total[5m])) / 1048576) or (sum(rate(pve_network_receive_bytes[5m]) + rate(pve_network_transmit_bytes[5m])) / 1048576) or (sum(rate(node_network_receive_bytes_total{device!="lo"}[5m]) + rate(node_network_transmit_bytes_total{device!="lo"}[5m])) / 1048576)"#
                        .to_string()
                }),
                services: Some(
                    self.queries
                        .services
                        .clone()
                        .unwrap_or_else(|| {
                            "(pve_up * on(id, instance) group_left(name) pve_guest_info) or probe_success"
                                .to_string()
                        }),
                ),
                services_is_default,
            }
        }
    }

    pub fn link_base_url(&self) -> Option<&str> {
        if let Some(public_url) = self.public_url.as_deref() {
            Some(public_url)
        } else if self.expose_links {
            Some(self.url.as_str())
        } else {
            None
        }
    }

    pub fn interval_ms(&self) -> u64 {
        time_range_window(&self.time_range).1
    }

    pub fn range_duration_ms(&self) -> i64 {
        time_range_window(&self.time_range).0
    }
}

pub fn time_range_window(range: &str) -> (i64, u64) {
    match range {
        "1h" => (3_600_000, 60_000),
        "3h" => (10_800_000, 150_000),
        "6h" => (21_600_000, 300_000),
        "12h" => (43_200_000, 600_000),
        "24h" => (86_400_000, 1_200_000),
        "7d" => (604_800_000, 8_400_000),
        _ => (21_600_000, 300_000),
    }
}

pub fn normalize_time_range(raw: Option<&str>) -> String {
    match raw.map(str::trim).unwrap_or("6h") {
        "1h" => "1h",
        "3h" => "3h",
        "6h" => "6h",
        "12h" => "12h",
        "24h" => "24h",
        "7d" => "7d",
        _ => "6h",
    }
    .to_string()
}

pub fn parse_dotenv(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").map(str::trim).unwrap_or(line);
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let val = val.trim();
        let unquoted = if (val.starts_with('"') && val.ends_with('"') && val.len() >= 2)
            || (val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2)
        {
            &val[1..val.len() - 1]
        } else {
            val
        };
        map.insert(key.to_string(), unquoted.to_string());
    }
    map
}

fn read_dotenv_file(path: &Path) -> HashMap<String, String> {
    match fs::read_to_string(path) {
        Ok(content) => parse_dotenv(&content),
        Err(_) => HashMap::new(),
    }
}

fn non_empty(val: Option<String>) -> Option<String> {
    val.and_then(|s| {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn parse_bool(val: Option<String>, default: bool) -> bool {
    match non_empty(val).as_deref().map(|s| s.to_ascii_lowercase()) {
        Some(ref s) if matches!(s.as_str(), "1" | "true" | "yes" | "on") => true,
        Some(ref s) if matches!(s.as_str(), "0" | "false" | "no" | "off") => false,
        _ => default,
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let dotenv = read_dotenv_file(Path::new(".env"));
        Self::from_lookup(|key| match env::var(key) {
            Ok(v) if !v.trim().is_empty() => Some(v),
            _ => dotenv.get(key).cloned(),
        })
    }

    pub fn from_lookup<F>(lookup: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let bind_raw = non_empty(lookup("BIND_ADDR")).unwrap_or_else(|| "0.0.0.0:3000".to_string());
        let bind_addr: SocketAddr = bind_raw
            .parse()
            .expect("BIND_ADDR must be a valid socket address, e.g. 0.0.0.0:3000");

        let poll_interval_secs = non_empty(lookup("GRAFANA_POLL_INTERVAL_SECS"))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30)
            .clamp(10, 3600);

        let timeout_secs = non_empty(lookup("GRAFANA_TIMEOUT_SECS"))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(8)
            .clamp(2, 30);

        let time_range = normalize_time_range(non_empty(lookup("GRAFANA_TIME_RANGE")).as_deref());

        let url = non_empty(lookup("GRAFANA_URL")).map(|s| s.trim_end_matches('/').to_string());
        let api_key = non_empty(lookup("GRAFANA_API_KEY"))
            .filter(|k| k != "glsa_your_service_account_token_here");

        let grafana = match (url, api_key) {
            (Some(url), Some(api_key)) => {
                let public_url = non_empty(lookup("GRAFANA_PUBLIC_URL"))
                    .map(|s| s.trim_end_matches('/').to_string());
                let expose_links = parse_bool(lookup("GRAFANA_EXPOSE_LINKS"), false);
                let org_id = non_empty(lookup("GRAFANA_ORG_ID")).and_then(|s| s.parse::<u64>().ok());
                let datasource_uid = non_empty(lookup("GRAFANA_DATASOURCE_UID"));
                let datasource_type = non_empty(lookup("GRAFANA_DATASOURCE_TYPE"))
                    .and_then(|s| {
                        let lower = s.to_ascii_lowercase();
                        if lower.contains("influx") {
                            Some("influxdb".to_string())
                        } else if lower.contains("prom") || lower.contains("victoria") {
                            Some("prometheus".to_string())
                        } else {
                            None
                        }
                    });
                let dashboard_tag = non_empty(lookup("GRAFANA_DASHBOARD_TAG"));
                let insecure_tls = parse_bool(lookup("GRAFANA_INSECURE_TLS"), false);

                let queries = GrafanaQueryOverrides {
                    cpu: non_empty(lookup("GRAFANA_QUERY_CPU")),
                    memory: non_empty(lookup("GRAFANA_QUERY_MEMORY")),
                    disk: non_empty(lookup("GRAFANA_QUERY_DISK")),
                    load: non_empty(lookup("GRAFANA_QUERY_LOAD")),
                    guests: non_empty(lookup("GRAFANA_QUERY_GUESTS")),
                    uptime: non_empty(lookup("GRAFANA_QUERY_UPTIME")),
                    network: non_empty(lookup("GRAFANA_QUERY_NETWORK")),
                    services: non_empty(lookup("GRAFANA_QUERY_SERVICES")),
                };

                Some(GrafanaConfig {
                    url,
                    api_key,
                    public_url,
                    expose_links,
                    org_id,
                    datasource_uid,
                    datasource_type,
                    dashboard_tag,
                    poll_interval_secs,
                    timeout_secs,
                    time_range: time_range.clone(),
                    insecure_tls,
                    queries,
                })
            }
            _ => None,
        };

        Self {
            bind_addr,
            grafana,
            grafana_time_range: time_range,
            grafana_poll_interval_secs: poll_interval_secs,
            grafana_timeout_secs: timeout_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dotenv_with_comments_quotes_and_equals() {
        let input = r#"
            # Comment line
            BIND_ADDR=127.0.0.1:4000
            GRAFANA_URL="https://grafana.example.com/"
            GRAFANA_API_KEY='glsa_abc=123=='
            export GRAFANA_EXPOSE_LINKS=true
            EMPTY=
        "#;

        let map = parse_dotenv(input);
        assert_eq!(map.get("BIND_ADDR").map(String::as_str), Some("127.0.0.1:4000"));
        assert_eq!(
            map.get("GRAFANA_URL").map(String::as_str),
            Some("https://grafana.example.com/")
        );
        assert_eq!(
            map.get("GRAFANA_API_KEY").map(String::as_str),
            Some("glsa_abc=123==")
        );
        assert_eq!(map.get("GRAFANA_EXPOSE_LINKS").map(String::as_str), Some("true"));
        assert_eq!(map.get("EMPTY").map(String::as_str), Some(""));
    }

    #[test]
    fn config_clamps_bounds_normalizes_urls_and_redacts_secret() {
        let mut env_map = HashMap::new();
        env_map.insert("BIND_ADDR".to_string(), "127.0.0.1:8080".to_string());
        env_map.insert(
            "GRAFANA_URL".to_string(),
            "https://grafana.internal.lan///".to_string(),
        );
        env_map.insert(
            "GRAFANA_API_KEY".to_string(),
            "glsa_super_secret_token".to_string(),
        );
        env_map.insert("GRAFANA_POLL_INTERVAL_SECS".to_string(), "2".to_string());
        env_map.insert("GRAFANA_TIMEOUT_SECS".to_string(), "999".to_string());
        env_map.insert("GRAFANA_TIME_RANGE".to_string(), "invalid".to_string());
        env_map.insert("GRAFANA_DATASOURCE_TYPE".to_string(), "influxdb".to_string());

        let cfg = AppConfig::from_lookup(|k| env_map.get(k).cloned());
        assert_eq!(cfg.bind_addr.to_string(), "127.0.0.1:8080");
        assert_eq!(cfg.grafana_poll_interval_secs, 10);
        assert_eq!(cfg.grafana_timeout_secs, 30);
        assert_eq!(cfg.grafana_time_range, "6h");

        let g = cfg.grafana.expect("grafana config should exist");
        assert_eq!(g.url, "https://grafana.internal.lan");
        assert_eq!(g.poll_interval_secs, 10);
        assert_eq!(g.timeout_secs, 30);
        assert_eq!(g.time_range, "6h");
        assert!(g.link_base_url().is_none());

        let dbg = format!("{g:?}");
        assert!(dbg.contains("<redacted>"));
        assert!(!dbg.contains("glsa_super_secret_token"));

        let influx_queries = g.resolved_queries("prometheus");
        assert!(influx_queries.cpu.contains("SELECT mean(\"cpu\")"));
    }

    #[test]
    fn unconfigured_when_key_or_url_missing_or_placeholder() {
        let mut env_map = HashMap::new();
        env_map.insert("GRAFANA_URL".to_string(), "https://grafana.urashyl.dev".to_string());
        let cfg = AppConfig::from_lookup(|k| env_map.get(k).cloned());
        assert!(cfg.grafana.is_none());

        env_map.insert(
            "GRAFANA_API_KEY".to_string(),
            "glsa_your_service_account_token_here".to_string(),
        );
        let cfg_placeholder = AppConfig::from_lookup(|k| env_map.get(k).cloned());
        assert!(cfg_placeholder.grafana.is_none());
    }
}
