use chrono::{DateTime, TimeZone, Utc};
use maud::{Markup, html};

use crate::{
    data::HOMELAB_SERVICES,
    grafana::{MetricSeries, TelemetryPoint, TelemetrySnapshot, TelemetryState},
};

pub fn format_metric_value(value: Option<f64>, unit: &str) -> String {
    let Some(v) = value else {
        return "—".to_string();
    };
    if !v.is_finite() {
        return "—".to_string();
    }
    match unit {
        "%" => format!("{v:.1}%"),
        "guests" => format!("{v:.0}"),
        "MB/s" => {
            if v >= 100.0 {
                format!("{v:.0} MB/s")
            } else {
                format!("{v:.2} MB/s")
            }
        }
        "" => format!("{v:.2}"),
        other => format!("{v:.1} {other}"),
    }
}

pub fn format_uptime_duration(seconds: Option<f64>) -> String {
    let Some(sec) = seconds else {
        return "—".to_string();
    };
    if !sec.is_finite() || sec < 0.0 {
        return "—".to_string();
    }
    let total = sec as u64;
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let mins = (total % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

pub fn format_iso_short(iso: Option<&str>) -> String {
    let Some(raw) = iso else {
        return "never".to_string();
    };
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        dt.with_timezone(&Utc).format("%Y-%m-%d %H:%M UTC").to_string()
    } else {
        raw.to_string()
    }
}

pub fn format_epoch_ms_short(ts_ms: i64) -> String {
    match Utc.timestamp_millis_opt(ts_ms).single() {
        Some(dt) => dt.format("%b %d · %H:%M UTC").to_string(),
        None => ts_ms.to_string(),
    }
}

struct ComputedPlot {
    polyline: String,
    area_path: String,
    last_x: f64,
    last_y: f64,
}

fn compute_plot(
    points: &[TelemetryPoint],
    x_min: f64,
    x_max: f64,
    y_top: f64,
    y_bottom: f64,
) -> Option<ComputedPlot> {
    if points.is_empty() {
        return None;
    }

    if points.len() == 1 {
        let mid_y = (y_top + y_bottom) * 0.5;
        let polyline = format!("{x_min:.1},{mid_y:.1} {x_max:.1},{mid_y:.1}");
        let area_path = format!(
            "M {x_min:.1} {y_bottom:.1} L {x_min:.1} {mid_y:.1} L {x_max:.1} {mid_y:.1} L {x_max:.1} {y_bottom:.1} Z"
        );
        return Some(ComputedPlot {
            polyline,
            area_path,
            last_x: x_max,
            last_y: mid_y,
        });
    }

    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    for p in points {
        if p.value < min_val {
            min_val = p.value;
        }
        if p.value > max_val {
            max_val = p.value;
        }
    }

    let span_val = (max_val - min_val).abs();
    let (v_lo, v_hi) = if span_val < 1e-6 {
        (min_val - 1.0, max_val + 1.0)
    } else {
        let pad = span_val * 0.1;
        (min_val - pad, max_val + pad)
    };
    let v_range = (v_hi - v_lo).max(1e-6);

    let first_ts = points.first().map(|p| p.ts_ms).unwrap_or(0);
    let last_ts = points.last().map(|p| p.ts_ms).unwrap_or(first_ts);
    let use_index_x = last_ts <= first_ts;

    let w = x_max - x_min;
    let h = y_bottom - y_top;

    let mut coords = Vec::with_capacity(points.len());
    for (idx, p) in points.iter().enumerate() {
        let nx = if use_index_x {
            idx as f64 / (points.len() - 1) as f64
        } else {
            (p.ts_ms - first_ts) as f64 / (last_ts - first_ts) as f64
        };
        let ny = ((p.value - v_lo) / v_range).clamp(0.0, 1.0);
        let x = x_min + nx * w;
        let y = y_bottom - ny * h;
        coords.push((x, y));
    }

    let polyline = coords
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");

    let first = coords[0];
    let last = coords[coords.len() - 1];
    let mut area_path = format!("M {:.1} {:.1}", first.0, y_bottom);
    for (x, y) in &coords {
        area_path.push_str(&format!(" L {x:.1} {y:.1}"));
    }
    area_path.push_str(&format!(" L {:.1} {:.1} Z", last.0, y_bottom));

    Some(ComputedPlot {
        polyline,
        area_path,
        last_x: last.0,
        last_y: last.1,
    })
}

pub fn render_sparkline(
    points: &[TelemetryPoint],
    series_key: &str,
    label: &str,
    accent_var: &str,
) -> Markup {
    let grad_id = format!("spark-grad-{series_key}");
    let plot = compute_plot(points, 4.0, 276.0, 8.0, 56.0);

    html! {
        svg
            class="sparkline-svg"
            viewBox="0 0 280 64"
            preserveAspectRatio="none"
            role="img"
            aria-label=(format!("{label} trend"))
            data-sparkline=(series_key)
        {
            defs {
                linearGradient id=(grad_id) x1="0" y1="0" x2="0" y2="1" {
                    stop offset="0%" stop-color=(accent_var) stop-opacity="0.22";
                    stop offset="100%" stop-color=(accent_var) stop-opacity="0.0";
                }
            }
            line x1="4" y1="32" x2="276" y2="32" stroke="var(--border)" stroke-width="1" stroke-dasharray="2 4";
            line x1="4" y1="56" x2="276" y2="56" stroke="var(--border)" stroke-width="1";
            @if let Some(p) = plot {
                path data-chart-area="" d=(p.area_path) fill=(format!("url(#{grad_id})"));
                polyline data-chart-line="" points=(p.polyline) fill="none" stroke=(accent_var) stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round";
                circle data-chart-dot="" cx=(format!("{:.1}", p.last_x)) cy=(format!("{:.1}", p.last_y)) r="2.75" fill=(accent_var);
            } @else {
                path data-chart-area="" d="" fill=(format!("url(#{grad_id})"));
                polyline data-chart-line="" points="" fill="none" stroke=(accent_var) stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round";
                circle data-chart-dot="" cx="-10" cy="-10" r="2.75" fill=(accent_var) class="hidden";
            }
        }
    }
}

pub fn render_metric_chart(series: &MetricSeries, accent_var: &str) -> Markup {
    let grad_id = format!("chart-grad-{}", series.key);
    let plot = compute_plot(&series.points, 8.0, 472.0, 14.0, 134.0);

    html! {
        div class="metric-chart-wrap" {
            svg
                class="metric-chart-svg"
                viewBox="0 0 480 148"
                preserveAspectRatio="none"
                role="img"
                aria-label=(format!("{} time series", series.label))
                data-chart=(series.key)
            {
                defs {
                    linearGradient id=(grad_id) x1="0" y1="0" x2="0" y2="1" {
                        stop offset="0%" stop-color=(accent_var) stop-opacity="0.24";
                        stop offset="100%" stop-color=(accent_var) stop-opacity="0.0";
                    }
                }
                line x1="8" y1="14" x2="472" y2="14" stroke="var(--border)" stroke-width="1" stroke-dasharray="3 4";
                line x1="8" y1="74" x2="472" y2="74" stroke="var(--border)" stroke-width="1" stroke-dasharray="3 4";
                line x1="8" y1="134" x2="472" y2="134" stroke="var(--border)" stroke-width="1";

                @if let Some(p) = plot {
                    path data-chart-area="" d=(p.area_path) fill=(format!("url(#{grad_id})"));
                    polyline data-chart-line="" points=(p.polyline) fill="none" stroke=(accent_var) stroke-width="2" stroke-linecap="round" stroke-linejoin="round";
                    circle data-chart-dot="" cx=(format!("{:.1}", p.last_x)) cy=(format!("{:.1}", p.last_y)) r="3.5" fill=(accent_var);
                } @else {
                    path data-chart-area="" d="" fill=(format!("url(#{grad_id})"));
                    polyline data-chart-line="" points="" fill="none" stroke=(accent_var) stroke-width="2" stroke-linecap="round" stroke-linejoin="round";
                    circle data-chart-dot="" cx="-10" cy="-10" r="3.5" fill=(accent_var) class="hidden";
                }
            }

            @if series.points.is_empty() {
                div class="chart-empty-overlay" data-chart-empty=(series.key) {
                    @if let Some(err) = &series.query_error {
                        span class="mono err-text" { "Query error: " (err) }
                    } @else {
                        span class="mono muted-text" { "Awaiting Grafana samples" }
                    }
                }
            } @else {
                div class="chart-empty-overlay hidden" data-chart-empty=(series.key) {
                    span class="mono muted-text" { "Awaiting Grafana samples" }
                }
            }
        }
    }
}

pub fn render_topology_diagram(snapshot: &TelemetrySnapshot) -> Markup {
    let grafana_color = match snapshot.state {
        TelemetryState::Connected => "var(--ok)",
        TelemetryState::Degraded => "var(--warn)",
        TelemetryState::Error => "var(--err)",
        TelemetryState::Unconfigured => "var(--muted)",
    };

    let probes_up = snapshot
        .service_probes
        .iter()
        .filter(|p| p.up == Some(true))
        .count();
    let probes_known = snapshot
        .service_probes
        .iter()
        .filter(|p| p.up.is_some())
        .count();
    let service_sub = if probes_known > 0 {
        format!("{probes_up}/{probes_known} probes healthy")
    } else {
        format!("{} core services", HOMELAB_SERVICES.len())
    };

    let pve_sub = match (snapshot.metrics.cpu.current, snapshot.metrics.memory.current) {
        (Some(c), Some(m)) => format!("CPU {c:.0}% · RAM {m:.0}%"),
        _ => "LXC + KVM isolation".to_string(),
    };

    html! {
        div class="topology-card" {
            div class="topology-head" {
                span class="eyebrow" { "Architecture // Request & Telemetry Flow" }
                span class="mono muted-text" style="font-size:0.75rem" {
                    "Edge TLS → Proxmox VE → Exporters → Grafana API → Axum SSE"
                }
            }
            svg class="topology-svg" viewBox="0 0 880 210" role="img" aria-label="Homelab network and Grafana telemetry architecture" {
                defs {
                    marker id="arrow-muted" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse" {
                        path d="M 0 1 L 8 5 L 0 9 z" fill="var(--muted)";
                    }
                    marker id="arrow-accent" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse" {
                        path d="M 0 1 L 8 5 L 0 9 z" fill="var(--accent)";
                    }
                }

                // Connection paths
                line x1="176" y1="68" x2="226" y2="68" stroke="var(--border-strong)" stroke-width="1.5" marker-end="url(#arrow-muted)";
                line x1="396" y1="68" x2="446" y2="68" stroke="var(--border-strong)" stroke-width="1.5" marker-end="url(#arrow-muted)";
                line x1="636" y1="68" x2="686" y2="68" stroke="var(--border-strong)" stroke-width="1.5" marker-end="url(#arrow-muted)";

                // Telemetry feedback loop (bottom path)
                path d="M 541 112 L 541 154 L 686 154" fill="none" stroke="var(--accent)" stroke-width="1.5" stroke-dasharray="4 4" marker-end="url(#arrow-accent)";
                path d="M 775 112 L 775 128" fill="none" stroke="var(--ok)" stroke-width="1.5" stroke-dasharray="3 3";

                // Node 1: Edge Proxy
                g transform="translate(16, 26)" {
                    rect width="160" height="84" rx="4" fill="var(--surface-alt)" stroke="var(--border-strong)";
                    text x="14" y="26" fill="var(--accent)" font-family="IBM Plex Mono, monospace" font-size="10" letter-spacing="1" { "01 / EDGE" }
                    text x="14" y="48" fill="var(--ink)" font-family="Instrument Sans, sans-serif" font-size="14" font-weight="600" { "Caddy TLS Proxy" }
                    text x="14" y="68" fill="var(--muted)" font-family="IBM Plex Mono, monospace" font-size="11" { "HTTPS · Reverse Proxy" }
                }

                // Node 2: Proxmox Host
                g transform="translate(226, 26)" {
                    rect width="170" height="84" rx="4" fill="var(--surface-alt)" stroke="var(--border-strong)";
                    text x="14" y="26" fill="var(--accent)" font-family="IBM Plex Mono, monospace" font-size="10" letter-spacing="1" { "02 / HYPERVISOR" }
                    text x="14" y="48" fill="var(--ink)" font-family="Instrument Sans, sans-serif" font-size="14" font-weight="600" { "Proxmox VE Host" }
                    text x="14" y="68" fill="var(--muted)" font-family="IBM Plex Mono, monospace" font-size="11" { (pve_sub) }
                }

                // Node 3: Workloads
                g transform="translate(446, 26)" {
                    rect width="190" height="84" rx="4" fill="var(--surface-alt)" stroke="var(--border-strong)";
                    text x="14" y="26" fill="var(--accent)" font-family="IBM Plex Mono, monospace" font-size="10" letter-spacing="1" { "03 / WORKLOADS" }
                    text x="14" y="48" fill="var(--ink)" font-family="Instrument Sans, sans-serif" font-size="14" font-weight="600" { "LXC & KVM Stack" }
                    text x="14" y="68" fill="var(--muted)" font-family="IBM Plex Mono, monospace" font-size="11" { (service_sub) }
                }

                // Node 4: Grafana API
                g transform="translate(686, 26)" {
                    rect width="178" height="84" rx="4" fill="var(--surface-alt)" stroke=(grafana_color);
                    circle cx="158" cy="22" r="4" fill=(grafana_color);
                    text x="14" y="26" fill="var(--accent)" font-family="IBM Plex Mono, monospace" font-size="10" letter-spacing="1" { "04 / GRAFANA" }
                    text x="14" y="48" fill="var(--ink)" font-family="Instrument Sans, sans-serif" font-size="14" font-weight="600" { "Grafana HTTP API" }
                    text x="14" y="68" fill="var(--muted)" font-family="IBM Plex Mono, monospace" font-size="11" { "State: " (snapshot.state.label()) }
                }

                // Node 5: urashyl.dev Axum
                g transform="translate(686, 128)" {
                    rect width="178" height="62" rx="4" fill="var(--surface)" stroke="var(--accent)";
                    text x="14" y="25" fill="var(--accent)" font-family="IBM Plex Mono, monospace" font-size="10" letter-spacing="1" { "05 / PORTFOLIO" }
                    text x="14" y="46" fill="var(--ink)" font-family="IBM Plex Mono, monospace" font-size="12" font-weight="600" { "urashyl.dev · Axum SSE" }
                }

                // Caption strip
                text x="16" y="160" fill="var(--muted)" font-family="IBM Plex Mono, monospace" font-size="11" {
                    "Telemetry poll: Bearer token stays server-side · Browser receives sanitized JSON + SSE frames"
                }
            }
        }
    }
}
