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
        Some(dt) => dt.format("%b %-d, %H:%M UTC").to_string(),
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

/// Uptime phrased for a sentence: "41 days", "6 hours", "12 minutes".
pub fn format_uptime_prose(seconds: Option<f64>) -> Option<String> {
    let sec = seconds.filter(|s| s.is_finite() && *s >= 0.0)? as u64;
    let plural = |n: u64, unit: &str| {
        if n == 1 {
            format!("1 {unit}")
        } else {
            format!("{n} {unit}s")
        }
    };
    Some(if sec >= 86_400 {
        plural(sec / 86_400, "day")
    } else if sec >= 3_600 {
        plural(sec / 3_600, "hour")
    } else {
        plural(sec / 60, "minute")
    })
}

/// Human phrase for a Grafana time range like "6h" or "7d".
pub fn time_range_phrase(range: &str) -> &'static str {
    match range {
        "1h" => "hour",
        "3h" => "3 hours",
        "12h" => "12 hours",
        "24h" => "24 hours",
        "7d" => "week",
        _ => "6 hours",
    }
}

/// One-sentence summary of the home server shown under the hero trace.
pub fn live_caption(snapshot: &TelemetrySnapshot) -> String {
    let m = &snapshot.metrics;
    if m.cpu.points.is_empty() && m.cpu.current.is_none() {
        return "Live metrics from the homelab are unavailable right now.".to_string();
    }
    let mut parts = Vec::new();
    if let Some(cpu) = m.cpu.current {
        parts.push(format!("{cpu:.0}% CPU"));
    }
    if let Some(mem) = m.memory.current {
        parts.push(format!("{mem:.0}% memory"));
    }
    let mut out = format!(
        "CPU on the homelab host over the past {}",
        time_range_phrase(&snapshot.time_range)
    );
    if !parts.is_empty() {
        out.push_str(". Now: ");
        out.push_str(&parts.join(", "));
    }
    if let Some(up) = format_uptime_prose(m.uptime_seconds) {
        out.push_str(&format!(", up {up}"));
    }
    out.push('.');
    out
}

/// Wide, axis-free trace used under the home page intro.
pub fn render_trace(series: &MetricSeries) -> Markup {
    let plot = compute_plot(&series.points, 0.0, 1000.0, 10.0, 110.0);
    html! {
        svg
            class="trace-svg"
            viewBox="0 0 1000 120"
            preserveAspectRatio="none"
            role="img"
            aria-label=(format!("{} over time", series.label))
            data-trace=(series.key)
        {
            line class="trace-base" x1="0" y1="110" x2="1000" y2="110" vector-effect="non-scaling-stroke" {}
            @if let Some(p) = &plot {
                path class="trace-area" data-chart-area="" d=(p.area_path) {}
                polyline class="trace-line" data-chart-line="" points=(p.polyline) vector-effect="non-scaling-stroke" {}
            } @else {
                path class="trace-area" data-chart-area="" d="" {}
                polyline class="trace-line" data-chart-line="" points="" vector-effect="non-scaling-stroke" {}
            }
        }
    }
}

pub fn render_metric_chart(series: &MetricSeries) -> Markup {
    let plot = compute_plot(&series.points, 8.0, 472.0, 14.0, 134.0);
    let empty_msg = match &series.query_error {
        Some(err) => format!("Query failed: {err}"),
        None => "No samples yet".to_string(),
    };

    html! {
        div class="chart-wrap" {
            svg
                class="chart-svg"
                viewBox="0 0 480 148"
                preserveAspectRatio="none"
                role="img"
                aria-label=(format!("{} over time", series.label))
                data-chart=(series.key)
            {
                line class="chart-grid" x1="8" y1="74" x2="472" y2="74" vector-effect="non-scaling-stroke" {}
                line class="chart-base" x1="8" y1="134" x2="472" y2="134" vector-effect="non-scaling-stroke" {}
                @if let Some(p) = &plot {
                    path class="chart-area" data-chart-area="" d=(p.area_path) {}
                    polyline class="chart-line" data-chart-line="" points=(p.polyline) vector-effect="non-scaling-stroke" {}
                    circle class="chart-dot" data-chart-dot="" cx=(format!("{:.1}", p.last_x)) cy=(format!("{:.1}", p.last_y)) r="3" {}
                } @else {
                    path class="chart-area" data-chart-area="" d="" {}
                    polyline class="chart-line" data-chart-line="" points="" vector-effect="non-scaling-stroke" {}
                    circle class="chart-dot hidden" data-chart-dot="" cx="-10" cy="-10" r="3" {}
                }
            }
            div class=(if series.points.is_empty() { "chart-empty" } else { "chart-empty hidden" }) data-chart-empty=(series.key) {
                span class=(if series.query_error.is_some() { "err-text" } else { "" }) { (empty_msg) }
            }
        }
    }
}

pub fn render_topology_diagram(snapshot: &TelemetrySnapshot) -> Markup {
    let grafana_stroke = match snapshot.state {
        TelemetryState::Connected => "var(--up)",
        TelemetryState::Degraded => "var(--warn)",
        TelemetryState::Error => "var(--down)",
        TelemetryState::Unconfigured => "var(--rule-strong)",
    };
    let probes_known = snapshot.service_probes.iter().filter(|p| p.up.is_some()).count();
    let probes_up = snapshot.service_probes.iter().filter(|p| p.up == Some(true)).count();
    let guests_line = if probes_known > 0 {
        format!("{probes_up} of {probes_known} services up")
    } else {
        format!("{} services", HOMELAB_SERVICES.len())
    };
    let host_line = match (snapshot.metrics.cpu.current, snapshot.metrics.memory.current) {
        (Some(c), Some(m)) => format!("CPU {c:.0}%, RAM {m:.0}%"),
        _ => "tartarus".to_string(),
    };

    // Boxes laid out on a single row; the observer path loops underneath.
    let node = |x: f64, w: f64, title: &str, sub: &str, stroke: &str| {
        html! {
            g transform=(format!("translate({x} 24)")) {
                rect width=(w) height="72" rx="10" fill="var(--surface)" stroke=(stroke) stroke-width="1.25" {}
                text class="topo-title" x="16" y="32" { (title) }
                text class="topo-sub" x="16" y="53" { (sub) }
            }
        }
    };

    html! {
        figure class="topology" {
            div class="topology-scroll" {
                svg class="topology-svg" viewBox="0 0 860 196" role="img" aria-label="How requests reach the homelab and how monitoring data reaches this site" {
                    defs {
                        marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse" {
                            path d="M0 1 9 5 0 9z" fill="var(--muted)" {}
                        }
                    }
                    (node(0.0, 150.0, "Internet", "HTTPS only", "var(--rule-strong)"))
                    (node(196.0, 170.0, "Caddy", "TLS, reverse proxy", "var(--rule-strong)"))
                    (node(412.0, 196.0, "Proxmox VE", &host_line, "var(--rule-strong)"))
                    (node(654.0, 206.0, "Containers and VMs", &guests_line, "var(--rule-strong)"))

                    line class="topo-link" x1="150" y1="60" x2="192" y2="60" marker-end="url(#arrow)" {}
                    line class="topo-link" x1="366" y1="60" x2="408" y2="60" marker-end="url(#arrow)" {}
                    line class="topo-link" x1="608" y1="60" x2="650" y2="60" marker-end="url(#arrow)" {}

                    path class="topo-link dashed" d="M757 96 V150 H560" marker-end="url(#arrow)" {}
                    g transform="translate(372 128)" {
                        rect width="184" height="44" rx="10" fill="var(--surface)" stroke=(grafana_stroke) stroke-width="1.25" {}
                        text class="topo-title" x="16" y="27" { "Prometheus, Grafana" }
                    }
                    path class="topo-link dashed" d="M372 150 H248" marker-end="url(#arrow)" {}
                    g transform="translate(96 128)" {
                        rect width="148" height="44" rx="10" fill="var(--accent-soft)" stroke="var(--accent)" stroke-width="1.25" {}
                        text class="topo-title" x="16" y="27" { "This website" }
                    }
                }
            }
            figcaption {
                "Requests come in through Caddy and land on a single Proxmox host. Prometheus scrapes the host through prometheus-pve-exporter; this site reads Grafana's API on a timer and never exposes the token or internal addresses."
            }
        }
    }
}
