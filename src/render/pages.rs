use maud::{Markup, html};

use crate::{
    data::{
        HOMELAB_SERVICES, HomelabService, PRINCIPLES, PROFILE, PROJECTS, Project, ProjectCategory,
        STACK_GROUPS, ServiceCategory, filter_services,
    },
    grafana::{MetricSeries, ServiceProbeStatus, TelemetrySnapshot, TelemetryState},
    render::{
        charts::{
            format_epoch_ms_short, format_iso_short, format_metric_value, format_uptime_duration,
            live_caption, render_metric_chart, render_topology_diagram, render_trace,
            time_range_phrase,
        },
        layout::{ActivePage, PageMeta, external_icon, github_icon, render_shell, state_class},
    },
};

fn probe_state(probe: Option<&ServiceProbeStatus>) -> (&'static str, &'static str) {
    match probe.and_then(|p| p.up) {
        Some(true) => ("probe-up", "Up"),
        Some(false) => ("probe-down", "Down"),
        None => ("probe-unknown", "Not monitored"),
    }
}

fn probe_badge(service_id: &str, probe: Option<&ServiceProbeStatus>) -> Markup {
    let (cls, txt) = probe_state(probe);
    html! {
        span class=(format!("probe {cls}")) data-service-badge=(service_id) { (txt) }
    }
}

fn ext_link(href: &str, label: &str) -> Markup {
    html! {
        a href=(href) rel="noopener noreferrer" { (label) (external_icon()) }
    }
}

fn work_row(project: &Project, heading_level_two: bool) -> Markup {
    html! {
        li class="work-row" {
            a class="work-link" href=(format!("/projects/{}", project.id)) {
                @if heading_level_two {
                    h2 class="work-name" { (project.name) }
                } @else {
                    h3 class="work-name" { (project.name) }
                }
                span class="work-desc" {
                    (project.tagline)
                    @if let Some(note) = project.status_note {
                        " " span class="work-note" { "(" (note.to_ascii_lowercase()) ")" }
                    }
                }
                span class="work-meta" { (project.language) ", " (project.updated_at) }
            }
        }
    }
}

pub fn render_home(snapshot: &TelemetrySnapshot) -> Markup {
    let show_live = snapshot.configured;

    let body = html! {
        section class="wrap hero" {
            h1 class="hero-title" { (PROFILE.headline) }
            p class="hero-sub" { (PROFILE.subheadline) }
            p class="hero-links" {
                a class="button" href="/projects" { "See my work" }
                a class="button button-quiet" href=(PROFILE.github_url) rel="noopener noreferrer" {
                    (github_icon()) " GitHub"
                }
            }
        }

        @if show_live {
            figure class="wrap live" data-live="" {
                (render_trace(&snapshot.metrics.cpu))
                figcaption {
                    span class=(format!("state-dot {}", state_class(snapshot.state))) data-state-dot="" {}
                    span data-live-caption="" { (live_caption(snapshot)) }
                    " "
                    a href="/telemetry" { "Full status" }
                }
            }
        }

        section class="wrap section" aria-labelledby="work-heading" {
            div class="section-head" {
                h2 id="work-heading" { "Selected work" }
                a href="/projects" { "All projects" }
            }
            ul class="work-list" {
                @for project in PROJECTS.iter().filter(|p| p.featured) {
                    (work_row(project, false))
                }
            }
        }

        section class="wrap section" aria-labelledby="home-lab-heading" {
            div class="section-head" {
                h2 id="home-lab-heading" { "Running at home" }
                a href="/homelab" { "How it's set up" }
            }
            p class="section-lead" {
                "Everything I'd otherwise pay a cloud company for runs on one Proxmox machine, each service in its own container. A few of them:"
            }
            ul class="service-list" {
                @for svc in HOMELAB_SERVICES.iter().filter(|s| matches!(s.category, ServiceCategory::Security | ServiceCategory::Storage | ServiceCategory::Media)).take(8) {
                    li {
                        span class="service-name" { (svc.name) }
                        span class="service-role" { (svc.role) }
                        @if snapshot.configured {
                            (probe_badge(svc.id, snapshot.probe_for_service(svc.id)))
                        }
                    }
                }
            }
        }
    };

    render_shell(
        PageMeta {
            title: "urashyl",
            description: "urashyl writes Rust services, hardens Linux systems, and runs a self-hosted Proxmox homelab.",
            path: "/",
            active: ActivePage::Home,
        },
        snapshot,
        body,
    )
}

pub fn render_projects(
    category: Option<ProjectCategory>,
    query: &str,
    projects: &[&Project],
    snapshot: &TelemetrySnapshot,
) -> Markup {
    let query = query.trim();
    let href_for = |cat: Option<ProjectCategory>| -> String {
        let mut params = Vec::new();
        if let Some(c) = cat {
            params.push(format!("category={}", c.as_str()));
        }
        if !query.is_empty() {
            params.push(format!("q={}", url_encode(query)));
        }
        if params.is_empty() {
            "/projects".to_string()
        } else {
            format!("/projects?{}", params.join("&"))
        }
    };

    let body = html! {
        section class="wrap page-head" {
            h1 { "Work" }
            p class="page-lead" {
                "Projects I've built and still maintain. Each one links to a write-up of how it works and why it's built that way."
            }
            nav class="filter" aria-label="Filter by category" {
                a href=(href_for(None)) aria-current=[category.is_none().then_some("page")] { "All" }
                @for cat in [ProjectCategory::Systems, ProjectCategory::Desktop] {
                    a href=(href_for(Some(cat))) aria-current=[(category == Some(cat)).then_some("page")] {
                        (cat.label())
                    }
                }
            }
            @if !query.is_empty() {
                p class="filter-note" {
                    "Matching “" (query) "”. "
                    a href=(match category { Some(c) => format!("/projects?category={}", c.as_str()), None => "/projects".to_string() }) { "Clear search" }
                }
            }
        }

        section class="wrap section-tight" {
            @if projects.is_empty() {
                div class="empty" {
                    p { "Nothing matches that filter." }
                    a class="button" href="/projects" { "Show all projects" }
                }
            } @else {
                ul class="work-list work-list-full" {
                    @for project in projects {
                        li class="work-entry" {
                            div class="work-entry-head" {
                                h2 { a href=(format!("/projects/{}", project.id)) { (project.name) } }
                                span class="work-meta" { (project.language) ", " (project.updated_at) }
                            }
                            p class="work-tagline" {
                                (project.tagline)
                                @if let Some(note) = project.status_note {
                                    " " span class="work-note" { "(" (note.to_ascii_lowercase()) ")" }
                                }
                            }
                            p class="work-body" { (project.description) }
                            p class="work-actions" {
                                a href=(format!("/projects/{}", project.id)) { "Read the write-up" }
                                (ext_link(project.repo_url, "Source"))
                                @if let Some(live) = project.live_url {
                                    (ext_link(live, "Live site"))
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    render_shell(
        PageMeta {
            title: "Work · urashyl",
            description: "Rust services, Linux security tooling, and desktop projects by urashyl.",
            path: "/projects",
            active: ActivePage::Projects,
        },
        snapshot,
        body,
    )
}

pub fn render_project_detail(
    project: &Project,
    prev: Option<&Project>,
    next: Option<&Project>,
    snapshot: &TelemetrySnapshot,
) -> Markup {
    let title = format!("{} · urashyl", project.name);
    let path = format!("/projects/{}", project.id);

    let body = html! {
        article class="wrap case" {
            p class="back" { a href="/projects" { "All work" } }

            header class="case-head" {
                h1 { (project.name) }
                p class="case-tagline" { (project.tagline) }
            }

            div class="case-grid" {
                div class="case-main prose" {
                    @if let Some(note) = project.status_note {
                        p class="callout" { (note) ". Use it to learn from, not to rely on." }
                    }
                    p { (project.overview) }

                    h2 { "How it's built" }
                    dl class="layers" {
                        @for layer in project.architecture {
                            div {
                                dt { (layer.layer) }
                                dd { (render_inline_code(layer.detail)) }
                            }
                        }
                    }

                    h2 { "What it does" }
                    ul {
                        @for item in project.highlights {
                            li { (item) }
                        }
                    }

                    h2 { "Decisions" }
                    ul {
                        @for decision in project.decisions {
                            li { (decision) }
                        }
                    }
                }

                aside class="case-side" {
                    dl class="facts" {
                        div { dt { "Role" } dd { (project.role) } }
                        div { dt { "Language" } dd { (project.language) } }
                        div { dt { "Updated" } dd { (project.updated_at) } }
                        div { dt { "Built with" } dd { (project.tags.join(", ")) } }
                    }
                    p class="case-links" {
                        a class="button" href=(project.repo_url) rel="noopener noreferrer" {
                            (github_icon()) " Source"
                        }
                        @if let Some(live) = project.live_url {
                            a class="button button-quiet" href=(live) rel="noopener noreferrer" {
                                "Visit " (live.trim_start_matches("https://")) (external_icon())
                            }
                        }
                    }
                }
            }

            nav class="pager" aria-label="More projects" {
                @if let Some(p) = prev {
                    a class="pager-prev" href=(format!("/projects/{}", p.id)) {
                        span class="pager-dir" { "Previous" }
                        span class="pager-name" { (p.name) }
                    }
                } @else {
                    span {}
                }
                @if let Some(n) = next {
                    a class="pager-next" href=(format!("/projects/{}", n.id)) {
                        span class="pager-dir" { "Next" }
                        span class="pager-name" { (n.name) }
                    }
                }
            }
        }
    };

    render_shell(
        PageMeta {
            title: &title,
            description: project.tagline,
            path: &path,
            active: ActivePage::Projects,
        },
        snapshot,
        body,
    )
}

/// Renders `backtick` spans in static copy as <code>.
fn render_inline_code(text: &str) -> Markup {
    html! {
        @for (i, part) in text.split('`').enumerate() {
            @if i % 2 == 1 { code { (part) } } @else { (part) }
        }
    }
}

fn service_entry(svc: &HomelabService, snapshot: &TelemetrySnapshot) -> Markup {
    let probe = snapshot.probe_for_service(svc.id);
    html! {
        li class="svc" {
            div class="svc-head" {
                h3 { (svc.name) }
                (probe_badge(svc.id, probe))
            }
            @if !svc.role.is_empty() { p class="svc-role" { (svc.role) } }
            @if !svc.description.is_empty() { p class="svc-desc" { (svc.description) } }
            p class="svc-meta" {
                (svc.runtime) "."
                @if !svc.exposure.is_empty() { " " (svc.exposure) "." }
            }
        }
    }
}

pub fn render_homelab(snapshot: &TelemetrySnapshot, category: Option<ServiceCategory>) -> Markup {
    let groups: Vec<ServiceCategory> = match category {
        Some(c) => vec![c],
        None => vec![
            ServiceCategory::Security,
            ServiceCategory::Storage,
            ServiceCategory::Media,
            ServiceCategory::Infrastructure,
            ServiceCategory::Machines,
        ],
    };
    let host_stats = match (snapshot.metrics.cpu.current, snapshot.metrics.memory.current) {
        (Some(cpu), Some(mem)) => Some(format!(
            "Right now the host is at {cpu:.0}% CPU and {mem:.0}% memory, and has been up for {}.",
            format_uptime_duration(snapshot.metrics.uptime_seconds)
        )),
        _ => None,
    };

    let body = html! {
        section class="wrap page-head" {
            h1 { "Homelab" }
            p class="page-lead" {
                "Everything runs on one Proxmox VE host called tartarus. Services get their own LXC container, heavier things like this website and mitch.pro get full VMs, and the only way in from outside is through nginx. The same box also holds the pool of desktop VMs that mitch.pro hands out on demand. "
                @if let Some(line) = &host_stats {
                    span data-homelab-stats="" { (line) }
                }
            }
        }

        section class="wrap section-tight" {
            (render_topology_diagram(snapshot))
        }

        @for cat in groups {
            section class="wrap section" {
                div class="section-head" {
                    h2 { (cat.label()) }
                    @if category.is_some() {
                        a href="/homelab" { "Show everything" }
                    }
                }
                ul class="svc-grid" {
                    @for svc in filter_services(Some(cat)) {
                        (service_entry(svc, snapshot))
                    }
                }
            }
        }
    };

    render_shell(
        PageMeta {
            title: "Homelab · urashyl",
            description: "The Proxmox VE homelab behind urashyl.dev: what runs on it and how it's wired.",
            path: "/homelab",
            active: ActivePage::Homelab,
        },
        snapshot,
        body,
    )
}

fn status_sentence(snapshot: &TelemetrySnapshot) -> String {
    match snapshot.state {
        TelemetryState::Connected => "Everything is reporting normally.".to_string(),
        TelemetryState::Degraded => {
            "Some metrics are missing, so parts of this page may be empty.".to_string()
        }
        TelemetryState::Error => "The last attempt to read from Grafana failed.".to_string(),
        TelemetryState::Unconfigured => {
            "Monitoring isn't connected on this server, so there's nothing to show yet.".to_string()
        }
    }
}

pub fn render_telemetry(snapshot: &TelemetrySnapshot) -> Markup {
    let show_setup = matches!(
        snapshot.state,
        TelemetryState::Unconfigured | TelemetryState::Error
    );
    let env_snippet = "\
GRAFANA_URL=https://grafana.example.com\n\
GRAFANA_API_KEY=glsa_...\n\
GRAFANA_POLL_INTERVAL_SECS=30\n\
GRAFANA_TIME_RANGE=6h";

    let body = html! {
        section class="wrap page-head" {
            h1 { "Server status" }
            p class="status-line" {
                span class=(format!("status-pill {}", state_class(snapshot.state))) data-status-pill="" {
                    span class="state-dot" {}
                    span data-status-label="" { (snapshot.state.label()) }
                }
                span data-status-sentence="" { (status_sentence(snapshot)) }
            }
            p class="page-lead small" {
                "Numbers from my Proxmox host over the past " (time_range_phrase(&snapshot.time_range)) ". "
                "The server reads Grafana every " (snapshot.poll_interval_secs) " seconds and pushes updates to this page as they arrive; "
                "your visit never triggers a query. Last read "
                span class="tabular" data-last-sync="" { (format_iso_short(snapshot.last_synced_at.as_deref())) }
                ". "
                span id="sse-indicator" class="sse" {}
            }
            p class=(if snapshot.last_error.is_some() { "error-banner" } else { "error-banner hidden" }) data-telemetry-error="" {
                @if let Some(err) = &snapshot.last_error { (err) }
            }
        }

        section class="wrap section-tight" aria-label="Metrics" {
            div class="metrics" {
                @for series in [
                    &snapshot.metrics.cpu,
                    &snapshot.metrics.memory,
                    &snapshot.metrics.disk,
                    &snapshot.metrics.load,
                    &snapshot.metrics.network,
                ] {
                    (render_metric_panel(series))
                }
                article class="metric metric-facts" {
                    div class="metric-head" {
                        h2 { "Uptime" }
                        span class="metric-now tabular" data-uptime-val="" {
                            (format_uptime_duration(snapshot.metrics.uptime_seconds))
                        }
                    }
                    dl class="facts compact" {
                        div { dt { "Grafana" } dd data-grafana-version="" {
                            @if let Some(ver) = &snapshot.grafana_version { "v" (ver) } @else { "—" }
                        } }
                        div { dt { "Data source" } dd data-active-datasource="" {
                            @if let Some(ds) = &snapshot.active_datasource { (ds.name) } @else { "—" }
                        } }
                        div { dt { "Dashboards" } dd class="tabular" data-dash-count="" { (snapshot.dashboards.len()) } }
                        div { dt { "Firing alerts" } dd class="tabular" data-alerts-count="" { (snapshot.firing_alerts_count()) } }
                    }
                }
            }
        }

        section class="wrap section two-up" {
            div {
                h2 { "Services" }
                table class="table" {
                    thead { tr { th { "Service" } th { "Checked by" } th { "Status" } } }
                    tbody data-probes-body="" {
                        @for probe in &snapshot.service_probes {
                            tr {
                                td { (probe.name) }
                                td class="muted" { (probe.source) }
                                td { (probe_badge(&probe.service_id, Some(probe))) }
                            }
                        }
                    }
                }
            }
            div {
                h2 { "Alerts" }
                div data-alerts-body="" {
                    @if snapshot.alerts.is_empty() {
                        p class="muted" {
                            @if snapshot.configured { "No alerts are firing." } @else { "Alerts appear here once monitoring is connected." }
                        }
                    } @else {
                        ul class="plain-list" {
                            @for alert in &snapshot.alerts {
                                li class="alert" {
                                    strong { (alert.name) }
                                    span class="muted" { " " (alert.severity) ", " (alert.state) }
                                    @if let Some(summary) = &alert.summary {
                                        p { (summary) }
                                    }
                                }
                            }
                        }
                    }
                }

                h2 class="spaced" { "Recent events" }
                div data-annotations-body="" {
                    @if snapshot.annotations.is_empty() {
                        p class="muted" { "Nothing logged in this window." }
                    } @else {
                        ul class="plain-list events" {
                            @for ann in snapshot.annotations.iter().take(8) {
                                li {
                                    time class="muted tabular" { (format_epoch_ms_short(ann.time_ms)) }
                                    span { (ann.text) }
                                }
                            }
                        }
                    }
                }
            }
        }

        @if !snapshot.dashboards.is_empty() {
            section class="wrap section" {
                h2 { "Dashboards" }
                ul class="plain-list dash-list" data-dashboards-body="" {
                    @for d in snapshot.dashboards.iter().take(12) {
                        li {
                            @if let Some(url) = &d.url {
                                (ext_link(url, &d.title))
                            } @else {
                                span { (d.title) }
                            }
                            @if let Some(folder) = &d.folder_title {
                                span class="muted" { " in " (folder) }
                            }
                        }
                    }
                }
            }
        }

        @if show_setup {
            section id="setup" class="wrap section" {
                details class="setup" open {
                    summary { "Connecting Grafana" }
                    div class="setup-body prose" {
                        p { "This page fills in once the server has a read-only Grafana token. To set one up:" }
                        ol {
                            li { "In Grafana, go to Administration, then Users and access, then Service accounts." }
                            li { "Create a service account with the Viewer role and add a token." }
                            li { "Put the URL and token in " code { ".env" } " next to the binary, then restart it." }
                        }
                        div class="code-head" {
                            span { code { ".env" } }
                            button type="button" id="copy-env-btn" class="link-button" { "Copy" }
                        }
                        pre id="env-snippet" { code { (env_snippet) } }
                        p class="muted" {
                            "Works with prometheus-pve-exporter, node_exporter, and Proxmox's InfluxDB output. Any query can be overridden with "
                            code { "GRAFANA_QUERY_*" } "."
                        }
                        @if !snapshot.diagnostics.is_empty() {
                            h3 { "What the server reported" }
                            ul class="diagnostics" data-diagnostics-list="" {
                                @for line in &snapshot.diagnostics {
                                    li { (line) }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    render_shell(
        PageMeta {
            title: "Server status · urashyl",
            description: "Live CPU, memory, disk and service status from the homelab that runs urashyl.dev.",
            path: "/telemetry",
            active: ActivePage::Telemetry,
        },
        snapshot,
        body,
    )
}

fn metric_title(series: &MetricSeries) -> &'static str {
    match series.key.as_str() {
        "cpu" => "CPU",
        "memory" => "Memory",
        "disk" => "Disk",
        "network" => "Network",
        "load" if series.unit == "guests" => "Running guests",
        "load" => "Load average",
        _ => "Metric",
    }
}

fn render_metric_panel(series: &MetricSeries) -> Markup {
    html! {
        article class="metric" data-metric-card=(series.key) {
            div class="metric-head" {
                h2 data-metric-title=(series.key) { (metric_title(series)) }
                span class="metric-now tabular" data-metric-value=(series.key) {
                    (format_metric_value(series.current, &series.unit))
                }
            }
            (render_metric_chart(series))
            p class="metric-range tabular" {
                "Low " span data-metric-min=(series.key) { (format_metric_value(series.min, &series.unit)) }
                ", average " span data-metric-avg=(series.key) { (format_metric_value(series.avg, &series.unit)) }
                ", high " span data-metric-max=(series.key) { (format_metric_value(series.max, &series.unit)) }
            }
        }
    }
}

pub fn render_about(snapshot: &TelemetrySnapshot) -> Markup {
    let body = html! {
        section class="wrap page-head about-head" {
            img src=(PROFILE.avatar_url) alt="" class="about-avatar" width="96" height="96";
            div {
                h1 { "About" }
                p class="page-lead" {
                    "I'm urashyl. I mostly write Rust and shell, run Arch on linux-hardened, and spend more time than is reasonable on my homelab."
                }
            }
        }

        section class="wrap section prose-wrap" {
            div class="prose" {
                p {
                    "My favorite kind of project is one where the security model is the interesting part: an Arch live system that leaves nothing behind, a sandbox that lets AI agents write code without touching the rest of my machine, a chat server where the host can't read the messages. I like getting those to work without making them miserable to use."
                }
                p {
                    "Most of what I build runs on hardware I own. That started as a way to stop paying for cloud storage and turned into a small Proxmox setup with monitoring, backups, and more containers than I'd planned."
                }
            }
        }

        section class="wrap section" aria-labelledby="principles-heading" {
            h2 id="principles-heading" { "How I like to build things" }
            div class="principles" {
                @for p in PRINCIPLES {
                    div {
                        h3 { (p.title) }
                        p { (p.summary) }
                    }
                }
            }
        }

        section class="wrap section" aria-labelledby="tools-heading" {
            h2 id="tools-heading" { "What I use" }
            dl class="stack" {
                @for group in STACK_GROUPS {
                    div {
                        dt { (group.title) }
                        dd { (group.items.join(", ")) }
                    }
                }
            }
        }

        section class="wrap section prose-wrap" aria-labelledby="site-heading" {
            div class="prose" {
                h2 id="site-heading" { "About this site" }
                p {
                    "It's a single Rust binary built with Axum and Maud, with the CSS and JavaScript compiled in. It's MIT licensed and the "
                    a href="https://github.com/urashyl/urashyl.dev" rel="noopener noreferrer" { "source is on GitHub" }
                    "."
                }
                p {
                    "The status numbers come from Grafana, read by the server on a fixed schedule with a read-only token. Internal addresses and private guests are stripped out before anything reaches your browser. The same data is available as JSON:"
                }
                ul class="endpoints" {
                    li { a href="/api/health" { code { "/api/health" } } }
                    li { a href="/api/telemetry" { code { "/api/telemetry" } } }
                    li { a href="/api/projects" { code { "/api/projects" } } }
                    li { a href="/api/services" { code { "/api/services" } } }
                    li { code { "/api/telemetry/stream" } " (server-sent events)" }
                }
            }
        }
    };

    render_shell(
        PageMeta {
            title: "About · urashyl",
            description: "Who urashyl is, how they like to build software, and how this site works.",
            path: "/about",
            active: ActivePage::About,
        },
        snapshot,
        body,
    )
}

pub fn render_not_found(path: &str, snapshot: &TelemetrySnapshot) -> Markup {
    let body = html! {
        section class="wrap page-head not-found" {
            h1 { "Page not found" }
            p class="page-lead" {
                "There's nothing at " code { (path) } ". It may have moved, or the link has a typo."
            }
            p class="hero-links" {
                a class="button" href="/" { "Go home" }
                a class="button button-quiet" href="/projects" { "Browse projects" }
            }
        }
    };

    render_shell(
        PageMeta {
            title: "Not found · urashyl",
            description: "Page not found.",
            path,
            active: ActivePage::NotFound,
        },
        snapshot,
        body,
    )
}

fn url_encode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else if byte == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
