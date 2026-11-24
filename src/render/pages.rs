use maud::{Markup, html};

use crate::{
    data::{
        EngineeringPrinciple, HOMELAB_SERVICES, PRINCIPLES, PROFILE, PROJECTS, Project,
        ProjectCategory, STACK_GROUPS, ServiceCategory, filter_services,
    },
    grafana::{MetricSeries, ServiceProbeStatus, TelemetrySnapshot, TelemetryState},
    render::{
        charts::{
            format_epoch_ms_short, format_iso_short, format_metric_value, format_uptime_duration,
            render_metric_chart, render_sparkline, render_topology_diagram,
        },
        layout::{ActivePage, arrow_up_right_icon, github_icon, render_shell, state_class},
    },
};

fn probe_badge_info(probe: Option<&ServiceProbeStatus>) -> (&'static str, &'static str) {
    match probe.and_then(|p| p.up) {
        Some(true) => ("probe-up", "UP"),
        Some(false) => ("probe-down", "DOWN"),
        None => ("probe-unknown", "UNMONITORED"),
    }
}

pub fn render_home(snapshot: &TelemetrySnapshot) -> Markup {
    let flagship = &PROJECTS[0];
    let featured: Vec<&Project> = PROJECTS
        .iter()
        .filter(|p| p.featured && p.id != flagship.id)
        .collect();
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

    let body = html! {
        section class="hero-section" {
            div class="container hero-grid" {
                div class="hero-copy" {
                    div class="identity-row" {
                        img class="avatar-thumb" src=(PROFILE.avatar_url) alt=(PROFILE.handle) width="24" height="24";
                        span class="mono" { "@" (PROFILE.handle) }
                        span class="sep" { "/" }
                        span class="mono accent-text" { (PROFILE.domain) }
                    }

                    h1 class="hero-title" {
                        "Rust backends, hardened Linux systems & "
                        em class="serif-italic" { "self-hosted infrastructure." }
                    }

                    p class="hero-lead" { (PROFILE.subheadline) }

                    div class="hero-cta-row" {
                        a href="/projects" class="btn-primary" {
                            span { "Explore Projects" }
                            (arrow_up_right_icon())
                        }
                        a href="/telemetry" class="btn-secondary" {
                            span { "Telemetry Console" }
                        }
                        a href=(PROFILE.github_url) target="_blank" rel="noreferrer" class="btn-ghost" {
                            (github_icon())
                            span { "GitHub" }
                        }
                    }

                    dl class="hero-specs" {
                        div {
                            dt { "PRIMARY STACK" }
                            dd { "Rust · Axum · Tokio · SQLite" }
                        }
                        div {
                            dt { "WORKSTATION" }
                            dd { (PROFILE.os) }
                        }
                        div {
                            dt { "INFRASTRUCTURE" }
                            dd { (PROFILE.hypervisor) " · Grafana API" }
                        }
                    }
                }

                aside class="telemetry-panel" data-home-telemetry="" {
                    div class="panel-header" {
                        div {
                            span class="eyebrow" { "Live Cluster Telemetry" }
                            p class="panel-sub mono" data-home-ds="" {
                                @if let Some(ds) = &snapshot.active_datasource {
                                    "Source: " (ds.name) " (" (ds.ds_type) ")"
                                } @else {
                                    "Grafana API · awaiting key"
                                }
                            }
                        }
                        a href="/telemetry" class=(format!("status-badge {}", state_class(snapshot.state))) data-home-state-badge="" {
                            span class="status-dot" {}
                            span data-home-state-label="" { (snapshot.state.label()) }
                        }
                    }

                    div class="spark-grid" {
                        @for (series, accent) in [
                            (&snapshot.metrics.cpu, "var(--accent)"),
                            (&snapshot.metrics.memory, "var(--ok)"),
                            (&snapshot.metrics.disk, "var(--warn)"),
                            (&snapshot.metrics.load, "var(--ink)"),
                        ] {
                            div class="spark-cell" data-metric-card=(series.key) {
                                div class="spark-top" {
                                    span class="mono spark-label" data-metric-short-label=(series.key) {
                                        @if series.unit == "guests" {
                                            "GUESTS"
                                        } @else {
                                            (series.key.to_ascii_uppercase())
                                        }
                                    }
                                    span class="mono tabular spark-val" data-metric-value=(series.key) {
                                        (format_metric_value(series.current, &series.unit))
                                    }
                                }
                                (render_sparkline(&series.points, &series.key, &series.label, accent))
                            }
                        }
                    }

                    div class="panel-footer" {
                        div class="panel-stat" {
                            span class="mono muted-text" { "UPTIME" }
                            span class="mono tabular" data-uptime-val="" {
                                @if snapshot.metrics.uptime_seconds.is_some() {
                                    (format_uptime_duration(snapshot.metrics.uptime_seconds))
                                } @else {
                                    (PROFILE.uptime) " target"
                                }
                            }
                        }
                        div class="panel-stat" {
                            span class="mono muted-text" { "ALERTS" }
                            span class="mono tabular" data-alerts-count="" {
                                (snapshot.firing_alerts_count()) " active"
                            }
                        }
                        div class="panel-stat" {
                            span class="mono muted-text" { "PROBES" }
                            span class="mono tabular" data-probes-summary="" {
                                @if probes_known > 0 {
                                    (probes_up) "/" (probes_known) " up"
                                } @else {
                                    (HOMELAB_SERVICES.len()) " services"
                                }
                            }
                        }
                        a href="/telemetry" class="mono panel-link" {
                            "Inspect →"
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container" {
                div class="section-head" {
                    div {
                        p class="eyebrow" { "01 // Systems & Repositories" }
                        h2 class="section-title" { "Selected Engineering Work" }
                    }
                    a href="/projects" class="section-more mono" {
                        "All " (PROJECTS.len()) " projects →"
                    }
                }

                article class="spotlight-card" {
                    div class="spotlight-body" {
                        div class="spotlight-meta" {
                            span class="pill-accent mono" { "Flagship · " (flagship.language) }
                            span class="mono muted-text" { "Updated " (flagship.updated_at) }
                        }
                        h3 class="spotlight-title" { (flagship.name) }
                        p class="spotlight-tagline" { (flagship.tagline) }
                        p class="card-desc" { (flagship.description) }
                        div class="spotlight-actions" {
                            a href=(format!("/projects/{}", flagship.id)) class="btn-primary btn-sm" {
                                "Read Case Study"
                            }
                            a href=(flagship.repo_url) target="_blank" rel="noreferrer" class="btn-ghost btn-sm" {
                                (github_icon())
                                span { "Repository" }
                            }
                        }
                    }
                    div class="spotlight-highlights" {
                        p class="eyebrow" { "Architecture Highlights" }
                        ul class="clean-list" {
                            @for h in flagship.highlights {
                                li { (h) }
                            }
                        }
                        div class="tag-row" {
                            @for tag in flagship.tags {
                                span class="tag" { (tag) }
                            }
                        }
                    }
                }

                div class="grid-3" {
                    @for project in &featured {
                        article class="card" {
                            div {
                                div class="card-top" {
                                    div {
                                        span class="mono muted-text card-kicker" {
                                            (project.category.label()) " · " (project.language)
                                        }
                                        h3 class="card-title" {
                                            a href=(format!("/projects/{}", project.id)) { (project.name) }
                                        }
                                    }
                                    a href=(project.repo_url) target="_blank" rel="noreferrer" class="icon-btn" aria-label=(format!("{} on GitHub", project.name)) {
                                        (arrow_up_right_icon())
                                    }
                                }
                                @if let Some(note) = project.status_note {
                                    div class="warn-chip mono" { (note) }
                                }
                                p class="card-tagline" { (project.tagline) }
                                p class="card-desc" { (project.description) }
                            }
                            div class="card-footer" {
                                div class="tag-row" {
                                    @for tag in project.tags.iter().take(3) {
                                        span class="tag" { (tag) }
                                    }
                                }
                                a href=(format!("/projects/{}", project.id)) class="mono card-link" {
                                    "Case study →"
                                }
                            }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container" {
                div class="section-head" {
                    div {
                        p class="eyebrow" { "02 // Proxmox VE Infrastructure" }
                        h2 class="section-title" { "Self-Hosted Stack & Live Probes" }
                        p class="section-desc" {
                            "Isolated LXC containers and KVM virtual machines on Proxmox VE — secrets, passwords, cloud storage, photos, media, and notes."
                        }
                    }
                    a href="/homelab" class="section-more mono" {
                        "Architecture & matrix →"
                    }
                }

                div class="grid-3" {
                    @for svc in HOMELAB_SERVICES {
                        @let probe = snapshot.probe_for_service(svc.id);
                        @let (badge_cls, badge_txt) = probe_badge_info(probe);
                        div class="service-strip-card" {
                            div class="card-top" {
                                div {
                                    h3 class="card-title" { (svc.name) }
                                    p class="mono muted-text" style="margin:0.2rem 0 0;font-size:0.75rem" { (svc.role) }
                                }
                                span class=(format!("probe-badge mono {badge_cls}")) data-service-badge=(svc.id) {
                                    (badge_txt)
                                }
                            }
                            p class="card-desc" { (svc.description) }
                            div class="service-meta-line mono" {
                                span { (svc.runtime) }
                                span { (svc.exposure) }
                            }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container two-col-wide" {
                div {
                    div class="section-head" style="margin-bottom:1.25rem" {
                        div {
                            p class="eyebrow" { "03 // Engineering Approach" }
                            h2 class="section-title" { "Operating Principles" }
                        }
                        a href="/about" class="section-more mono" { "About & stack →" }
                    }
                    div class="principles-stack" {
                        @for EngineeringPrinciple { index, title, summary } in PRINCIPLES {
                            div class="principle-row" {
                                span class="mono principle-idx" { (index) }
                                div {
                                    h3 class="principle-title" { (title) }
                                    p class="card-desc" { (summary) }
                                }
                            }
                        }
                    }
                }

                aside class="activity-card" {
                    div class="panel-header" {
                        div {
                            p class="eyebrow" { "Grafana Event Feed" }
                            h3 class="card-title" style="margin-top:0.25rem" { "Recent Annotations & Status" }
                        }
                        a href="/telemetry" class="mono panel-link" { "Telemetry →" }
                    }
                    @if snapshot.annotations.is_empty() {
                        div class="empty-note" {
                            p class="card-desc" {
                                @if snapshot.configured {
                                    "Connected to Grafana — no recent annotations returned in the configured window."
                                } @else {
                                    "Grafana API is currently in standby. Configure GRAFANA_URL and GRAFANA_API_KEY in `.env` to stream live Proxmox metrics, alerts, and annotations."
                                }
                            }
                            a href="/telemetry#setup" class="btn-secondary btn-sm" style="margin-top:0.75rem" {
                                "View Grafana Setup"
                            }
                        }
                    } @else {
                        ul class="annotation-list" {
                            @for ann in snapshot.annotations.iter().take(5) {
                                li class="annotation-item" {
                                    div class="mono muted-text" style="font-size:0.72rem" {
                                        (format_epoch_ms_short(ann.time_ms))
                                    }
                                    p class="annotation-text" { (ann.text) }
                                    @if !ann.tags.is_empty() {
                                        div class="tag-row" {
                                            @for tag in &ann.tags {
                                                span class="tag" { (tag) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    render_shell(
        "urashyl.dev — Rust Systems, Hardened Linux & Live Homelab Telemetry",
        "Portfolio of urashyl: Rust/Axum platforms, Arch Linux security engineering, Wayland desktops, and Proxmox VE homelab observed via Grafana.",
        ActivePage::Home,
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
    let q_suffix = if query.trim().is_empty() {
        String::new()
    } else {
        format!("&q={}", url_encode(query.trim()))
    };

    let tab_href = |cat: Option<ProjectCategory>| -> String {
        match cat {
            None => {
                if query.trim().is_empty() {
                    "/projects".to_string()
                } else {
                    format!("/projects?q={}", url_encode(query.trim()))
                }
            }
            Some(c) => format!("/projects?category={}{}", c.as_str(), q_suffix),
        }
    };

    let body = html! {
        section class="page-head-band" {
            div class="container" {
                p class="eyebrow" { "01 // Systems & Repositories" }
                h1 class="page-title" { "Projects & Architectural Case Studies" }
                p class="page-lead" {
                    "Rust network services, amnesic Arch Linux engineering, container sandboxing, and Wayland desktop tooling."
                }

                div class="filter-toolbar" {
                    nav class="filter-pills" aria-label="Project categories" {
                        a href=(tab_href(None)) class=(if category.is_none() { "filter-pill active" } else { "filter-pill" }) {
                            "All (" (PROJECTS.len()) ")"
                        }
                        @for cat in [ProjectCategory::Systems, ProjectCategory::Desktop] {
                            a
                                href=(tab_href(Some(cat)))
                                class=(if category == Some(cat) { "filter-pill active" } else { "filter-pill" })
                            {
                                (cat.label())
                            }
                        }
                    }

                    form method="get" action="/projects" class="search-form" {
                        @if let Some(cat) = category {
                            input type="hidden" name="category" value=(cat.as_str());
                        }
                        input
                            type="search"
                            name="q"
                            value=(query)
                            class="search-input"
                            placeholder="Filter by language, tag, or keyword..."
                            aria-label="Filter projects";
                        button type="submit" class="btn-secondary btn-sm" { "Filter" }
                        @if category.is_some() || !query.trim().is_empty() {
                            a href="/projects" class="btn-ghost btn-sm" { "Reset" }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container" {
                @if projects.is_empty() {
                    div class="empty-state-box" {
                        h2 class="card-title" { "No projects matched your filter" }
                        p class="card-desc" {
                            "Try clearing your search query or switching back to all categories."
                        }
                        a href="/projects" class="btn-primary btn-sm" style="margin-top:0.75rem" { "View All Projects" }
                    }
                } @else {
                    div class="grid-2" {
                        @for project in projects {
                            article class="card" {
                                div {
                                    div class="card-top" {
                                        div {
                                            span class="mono muted-text card-kicker" {
                                                (project.category.label()) " · " (project.language) " · " (project.updated_at)
                                            }
                                            h2 class="card-title" style="font-size:1.15rem" {
                                                a href=(format!("/projects/{}", project.id)) { (project.name) }
                                            }
                                        }
                                        a href=(project.repo_url) target="_blank" rel="noreferrer" class="icon-btn" aria-label=(format!("{} source on GitHub", project.name)) {
                                            (arrow_up_right_icon())
                                        }
                                    }

                                    @if let Some(note) = project.status_note {
                                        div class="warn-chip mono" { (note) }
                                    }

                                    p class="card-tagline" { (project.tagline) }
                                    p class="card-desc" { (project.description) }

                                    ul class="clean-list compact" {
                                        @for h in project.highlights {
                                            li { (h) }
                                        }
                                    }
                                }

                                div class="card-footer" {
                                    div class="tag-row" {
                                        @for tag in project.tags {
                                            span class="tag" { (tag) }
                                        }
                                    }
                                    a href=(format!("/projects/{}", project.id)) class="mono card-link" {
                                        "Case study →"
                                    }
                                }
                            }
                        }
                    }

                    div class="table-section" {
                        h2 class="eyebrow" style="margin-bottom:0.85rem" { "Repository Index" }
                        div class="table-wrap" {
                            table class="data-table" {
                                thead {
                                    tr {
                                        th { "Project" }
                                        th { "Category" }
                                        th { "Language" }
                                        th { "Updated" }
                                        th { "Links" }
                                    }
                                }
                                tbody {
                                    @for project in projects {
                                        tr {
                                            td {
                                                a href=(format!("/projects/{}", project.id)) class="mono table-title-link" { (project.name) }
                                                div class="muted-text" style="font-size:0.8rem" { (project.tagline) }
                                            }
                                            td class="mono" { (project.category.label()) }
                                            td class="mono" { (project.language) }
                                            td class="mono tabular" { (project.updated_at) }
                                            td class="mono" {
                                                div class="table-actions" {
                                                    a href=(format!("/projects/{}", project.id)) { "Case Study" }
                                                    span class="sep" { "·" }
                                                    a href=(project.repo_url) target="_blank" rel="noreferrer" { "GitHub" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    render_shell(
        "Projects — urashyl.dev",
        "All software systems, security tooling, and desktop projects by urashyl.",
        ActivePage::Projects,
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
    let body = html! {
        section class="page-head-band" {
            div class="container" {
                nav class="breadcrumb mono" aria-label="Breadcrumb" {
                    a href="/" { "Overview" }
                    span { "/" }
                    a href="/projects" { "Projects" }
                    span { "/" }
                    span { (project.name) }
                }

                div class="detail-header-grid" {
                    div {
                        @if let Some(note) = project.status_note {
                            div class="warn-chip mono" style="margin-bottom:0.75rem" { (note) }
                        }
                        h1 class="page-title" { (project.name) }
                        p class="detail-tagline" { (project.tagline) }
                        p class="page-lead" { (project.overview) }
                    }

                    aside class="spec-card" {
                        dl class="spec-list" {
                            div {
                                dt class="mono" { "CATEGORY" }
                                dd class="mono" { (project.category.label()) }
                            }
                            div {
                                dt class="mono" { "LANGUAGE" }
                                dd class="mono" { (project.language) }
                            }
                            div {
                                dt class="mono" { "ROLE" }
                                dd { (project.role) }
                            }
                            div {
                                dt class="mono" { "UPDATED" }
                                dd class="mono tabular" { (project.updated_at) }
                            }
                        }

                        div class="tag-row" style="margin-top:1rem" {
                            @for tag in project.tags {
                                span class="tag" { (tag) }
                            }
                        }

                        div class="spec-actions" {
                            a href=(project.repo_url) target="_blank" rel="noreferrer" class="btn-primary btn-sm" {
                                (github_icon())
                                span { "View Repository" }
                            }
                            @if let Some(live) = project.live_url {
                                a href=(live) target="_blank" rel="noreferrer" class="btn-secondary btn-sm" {
                                    span { "Live Domain" }
                                    (arrow_up_right_icon())
                                }
                            }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container" {
                p class="eyebrow" { "System Architecture" }
                h2 class="section-title" style="margin-bottom:1.25rem" { "Component Layers" }

                div class="arch-grid" {
                    @for layer in project.architecture {
                        div class="arch-card" {
                            h3 class="mono arch-layer-title" { (layer.layer) }
                            p class="card-desc" { (layer.detail) }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container grid-2" {
                div class="card" {
                    p class="eyebrow" { "Implementation Highlights" }
                    h2 class="card-title" style="margin-top:0.35rem" { "Core Capabilities" }
                    ul class="clean-list" {
                        @for item in project.highlights {
                            li { (item) }
                        }
                    }
                }

                div class="card" {
                    p class="eyebrow" { "Tradeoffs & Rationale" }
                    h2 class="card-title" style="margin-top:0.35rem" { "Engineering Decisions" }
                    ul class="clean-list" {
                        @for decision in project.decisions {
                            li { (decision) }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container detail-pager" {
                @if let Some(p) = prev {
                    a href=(format!("/projects/{}", p.id)) class="pager-link prev" {
                        span class="mono muted-text" { "← Previous" }
                        span class="pager-name mono" { (p.name) }
                    }
                } @else {
                    a href="/projects" class="pager-link prev" {
                        span class="mono muted-text" { "← Index" }
                        span class="pager-name mono" { "All Projects" }
                    }
                }

                @if let Some(n) = next {
                    a href=(format!("/projects/{}", n.id)) class="pager-link next" {
                        span class="mono muted-text" { "Next →" }
                        span class="pager-name mono" { (n.name) }
                    }
                } @else {
                    a href="/homelab" class="pager-link next" {
                        span class="mono muted-text" { "Continue →" }
                        span class="pager-name mono" { "Proxmox Homelab" }
                    }
                }
            }
        }
    };

    render_shell(
        &format!("{} — Projects — urashyl.dev", project.name),
        project.tagline,
        ActivePage::Projects,
        snapshot,
        body,
    )
}

pub fn render_homelab(
    snapshot: &TelemetrySnapshot,
    category: Option<ServiceCategory>,
) -> Markup {
    let services = filter_services(category);
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

    let body = html! {
        section class="page-head-band" {
            div class="container" {
                p class="eyebrow" { "02 // Self-Hosted Infrastructure" }
                h1 class="page-title" { "Proxmox VE Homelab" }
                p class="page-lead" {
                    "Production personal infrastructure running on Proxmox VE with LXC/KVM workload isolation, automated snapshots, reverse-proxy TLS, and live Grafana monitoring."
                }

                div class="kpi-strip" {
                    div class="kpi-card" {
                        span class="mono kpi-label" { "HYPERVISOR" }
                        strong class="kpi-val mono" { (PROFILE.hypervisor) }
                        span class="mono kpi-sub" { "LXC + KVM isolation · ZFS" }
                    }
                    div class="kpi-card" {
                        span class="mono kpi-label" { "LIVE HOST LOAD" }
                        strong class="kpi-val mono tabular" data-homelab-load="" {
                            @if let Some(cpu) = snapshot.metrics.cpu.current {
                                (format!("{cpu:.1}% CPU"))
                            } @else {
                                (PROFILE.uptime) " target"
                            }
                        }
                        span class="mono kpi-sub" data-homelab-mem="" {
                            @if let Some(mem) = snapshot.metrics.memory.current {
                                (format!("Memory {mem:.1}% · Uptime {}", format_uptime_duration(snapshot.metrics.uptime_seconds)))
                            } @else {
                                "Connect Grafana API for live host stats"
                            }
                        }
                    }
                    div class="kpi-card" {
                        span class="mono kpi-label" { "SERVICE MATRIX" }
                        strong class="kpi-val mono tabular" {
                            (HOMELAB_SERVICES.len()) " Core Services"
                        }
                        span class="mono kpi-sub" {
                            @if probes_known > 0 {
                                (probes_up) "/" (probes_known) " probed healthy"
                            } @else {
                                "Zero third-party lock-in"
                            }
                        }
                    }
                    div class="kpi-card" {
                        span class="mono kpi-label" { "GRAFANA TELEMETRY" }
                        strong class=(format!("kpi-val mono {}", state_class(snapshot.state))) {
                            (snapshot.state.label())
                        }
                        a href="/telemetry" class="mono kpi-sub panel-link" { "Open Live Console →" }
                    }
                }
            }
        }

        section class="band" {
            div class="container" {
                (render_topology_diagram(snapshot))
            }
        }

        section class="band" {
            div class="container" {
                div class="section-head" {
                    div {
                        p class="eyebrow" { "Workloads & Probes" }
                        h2 class="section-title" { "Running Services" }
                    }

                    nav class="filter-pills" aria-label="Service categories" {
                        a href="/homelab" class=(if category.is_none() { "filter-pill active" } else { "filter-pill" }) {
                            "All (" (HOMELAB_SERVICES.len()) ")"
                        }
                        @for cat in [ServiceCategory::Security, ServiceCategory::Storage, ServiceCategory::Media] {
                            a
                                href=(format!("/homelab?category={}", cat.as_str()))
                                class=(if category == Some(cat) { "filter-pill active" } else { "filter-pill" })
                            {
                                (cat.label())
                            }
                        }
                    }
                }

                div class="grid-3" {
                    @for svc in services {
                        @let probe = snapshot.probe_for_service(svc.id);
                        @let (badge_cls, badge_txt) = probe_badge_info(probe);
                        article class="card" {
                            div {
                                div class="card-top" {
                                    div {
                                        span class="mono muted-text card-kicker" { (svc.category.label()) }
                                        h3 class="card-title" { (svc.name) }
                                        p class="mono muted-text" style="margin:0.2rem 0 0;font-size:0.75rem" { (svc.role) }
                                    }
                                    span class=(format!("probe-badge mono {badge_cls}")) data-service-badge=(svc.id) {
                                        (badge_txt)
                                    }
                                }

                                p class="card-desc" { (svc.description) }

                                @if let Some(p) = probe {
                                    @if let Some(target) = &p.target {
                                        p class="mono probe-target" data-service-target=(svc.id) {
                                            "Probe target: " (target)
                                        }
                                    } @else {
                                        p class="mono probe-target hidden" data-service-target=(svc.id) {}
                                    }
                                } @else {
                                    p class="mono probe-target hidden" data-service-target=(svc.id) {}
                                }

                                ul class="clean-list compact" {
                                    @for h in svc.highlights {
                                        li { (h) }
                                    }
                                }
                            }

                            div class="service-foot-meta mono" {
                                span { (svc.runtime) }
                                span { (svc.exposure) }
                            }
                        }
                    }
                }
            }
        }
    };

    render_shell(
        "Homelab — urashyl.dev",
        "Proxmox VE homelab topology, self-hosted service inventory, and live Grafana probe status.",
        ActivePage::Homelab,
        snapshot,
        body,
    )
}

pub fn render_telemetry(snapshot: &TelemetrySnapshot) -> Markup {
    let show_setup_open = matches!(
        snapshot.state,
        TelemetryState::Unconfigured | TelemetryState::Error
    );

    let env_snippet = "\
GRAFANA_URL=https://grafana.urashyl.dev\n\
GRAFANA_API_KEY=glsa_your_service_account_token_here\n\
GRAFANA_POLL_INTERVAL_SECS=30\n\
GRAFANA_TIME_RANGE=6h";

    let body = html! {
        section class="page-head-band" {
            div class="container" {
                div class="telemetry-top-row" {
                    div {
                        p class="eyebrow" { "03 // Observability Console" }
                        h1 class="page-title" { "Live Grafana Telemetry" }
                        p class="page-lead" {
                            "Scheduled backend-polled metrics, unified alerts, service probes, and annotations served from server cache and Server-Sent Events — visitors never trigger upstream Grafana API queries."
                        }
                    }

                    div class="telemetry-controls" {
                        span class="status-badge state-standby mono" {
                            "Cached · polled every " (snapshot.poll_interval_secs) "s"
                        }
                        span id="sse-indicator" class="mono muted-text" style="font-size:0.75rem" {
                            "SSE: connecting…"
                        }
                    }
                }

                div class="telemetry-meta-bar" {
                    div class="meta-cell" {
                        span class="mono kpi-label" { "STATUS" }
                        span class=(format!("status-badge {}", state_class(snapshot.state))) data-telemetry-banner-badge="" {
                            span class="status-dot" {}
                            span data-telemetry-banner-label="" { (snapshot.state.label()) }
                        }
                    }
                    div class="meta-cell" {
                        span class="mono kpi-label" { "GRAFANA" }
                        span class="mono" data-grafana-version="" {
                            @if let Some(ver) = &snapshot.grafana_version {
                                "v" (ver)
                                @if let Some(org) = &snapshot.org_name {
                                    " · " (org)
                                }
                            } @else {
                                "Not connected"
                            }
                        }
                    }
                    div class="meta-cell" {
                        span class="mono kpi-label" { "DATASOURCE" }
                        span class="mono" data-active-datasource="" {
                            @if let Some(ds) = &snapshot.active_datasource {
                                (ds.name) " (" (ds.ds_type) ")"
                            } @else {
                                "Auto-discovery"
                            }
                        }
                    }
                    div class="meta-cell" {
                        span class="mono kpi-label" { "WINDOW / POLL" }
                        span class="mono tabular" {
                            (snapshot.time_range) " · every " (snapshot.poll_interval_secs) "s"
                        }
                    }
                    div class="meta-cell" {
                        span class="mono kpi-label" { "LAST SYNCED" }
                        span class="mono tabular" data-last-sync="" {
                            (format_iso_short(snapshot.last_synced_at.as_deref()))
                        }
                    }
                }

                @if let Some(err) = &snapshot.last_error {
                    div class="error-banner mono" data-telemetry-error="" {
                        "Sync Error: " (err)
                    }
                } @else {
                    div class="error-banner mono hidden" data-telemetry-error="" {}
                }
            }
        }

        section class="band" {
            div class="container" {
                p class="eyebrow" style="margin-bottom:1rem" { "Cluster Metrics" }
                div class="metrics-grid" {
                    @for (series, accent) in [
                        (&snapshot.metrics.cpu, "var(--accent)"),
                        (&snapshot.metrics.memory, "var(--ok)"),
                        (&snapshot.metrics.disk, "var(--warn)"),
                        (&snapshot.metrics.load, "var(--ink)"),
                        (&snapshot.metrics.network, "var(--ok)"),
                    ] {
                        (render_metric_panel(series, accent))
                    }

                    article class="metric-card" {
                        div class="metric-card-head" {
                            div {
                                span class="mono kpi-label" { "NODE & GRAFANA HEALTH" }
                                h2 class="metric-card-title" { "Host Uptime" }
                            }
                            span class="mono tabular metric-current" data-uptime-val="" {
                                (format_uptime_duration(snapshot.metrics.uptime_seconds))
                            }
                        }

                        dl class="uptime-spec-grid" {
                            div {
                                dt class="mono" { "DATABASE" }
                                dd class="mono" data-grafana-db="" {
                                    (snapshot.grafana_database.as_deref().unwrap_or("—"))
                                }
                            }
                            div {
                                dt class="mono" { "DATASOURCES" }
                                dd class="mono tabular" data-ds-count="" {
                                    (snapshot.datasources.len())
                                }
                            }
                            div {
                                dt class="mono" { "DASHBOARDS" }
                                dd class="mono tabular" data-dash-count="" {
                                    (snapshot.dashboards.len())
                                }
                            }
                            div {
                                dt class="mono" { "ALERTS" }
                                dd class="mono tabular" data-alerts-count="" {
                                    (snapshot.firing_alerts_count())
                                }
                            }
                        }

                        div class="metric-stats-row mono tabular" {
                            span { "Target SLA: " (PROFILE.uptime) }
                            span { "OS: Arch Linux" }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container grid-2" {
                div class="card" {
                    div class="panel-header" style="margin-bottom:0.75rem" {
                        div {
                            p class="eyebrow" { "Blackbox / Exporter Probes" }
                            h2 class="card-title" { "Service Reachability" }
                        }
                    }
                    div class="table-wrap" {
                        table class="data-table" {
                            thead {
                                tr {
                                    th { "Service" }
                                    th { "Source" }
                                    th { "Target" }
                                    th { "Status" }
                                }
                            }
                            tbody data-probes-body="" {
                                @for probe in &snapshot.service_probes {
                                    @let (cls, txt) = probe_badge_info(Some(probe));
                                    tr {
                                        td class="mono" { (probe.name) }
                                        td class="mono muted-text" { (probe.source) }
                                        td class="mono muted-text" { (probe.target.as_deref().unwrap_or("—")) }
                                        td {
                                            span class=(format!("probe-badge mono {cls}")) { (txt) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div class="card" {
                    div class="panel-header" style="margin-bottom:0.75rem" {
                        div {
                            p class="eyebrow" { "Grafana Unified Alerting" }
                            h2 class="card-title" { "Active Alerts" }
                        }
                    }
                    div data-alerts-body="" {
                        @if snapshot.alerts.is_empty() {
                            div class="empty-note" {
                                p class="card-desc" {
                                    @if snapshot.configured {
                                        "No alerts reported by Grafana Alertmanager."
                                    } @else {
                                        "Waiting for Grafana API connection."
                                    }
                                }
                            }
                        } @else {
                            ul class="alert-list" {
                                @for alert in &snapshot.alerts {
                                    li class="alert-row" {
                                        div class="alert-top" {
                                            strong class="mono" { (alert.name) }
                                            span class="mono alert-sev" { (alert.severity) " · " (alert.state) }
                                        }
                                        @if let Some(summary) = &alert.summary {
                                            p class="card-desc" { (summary) }
                                        }
                                        @if !alert.labels.is_empty() {
                                            div class="tag-row" {
                                                @for (k, v) in &alert.labels {
                                                    span class="tag" { (k) "=" (v) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container grid-2" {
                div class="card" {
                    p class="eyebrow" { "Grafana Inventory" }
                    h2 class="card-title" style="margin:0.25rem 0 0.9rem" { "Dashboards & Datasources" }

                    div data-dashboards-body="" {
                        @if snapshot.dashboards.is_empty() {
                            p class="card-desc" { "No dashboards indexed yet." }
                        } @else {
                            ul class="dash-list" {
                                @for d in snapshot.dashboards.iter().take(10) {
                                    li class="dash-row" {
                                        div {
                                            @if let Some(url) = &d.url {
                                                a href=(url) target="_blank" rel="noreferrer" class="mono table-title-link" {
                                                    (d.title)
                                                }
                                            } @else {
                                                span class="mono" { (d.title) }
                                            }
                                            @if let Some(folder) = &d.folder_title {
                                                span class="mono muted-text" style="margin-left:0.5rem;font-size:0.75rem" {
                                                    "[" (folder) "]"
                                                }
                                            }
                                        }
                                        @if !d.tags.is_empty() {
                                            div class="tag-row" style="margin-top:0.35rem" {
                                                @for tag in &d.tags {
                                                    span class="tag" { (tag) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    @if !snapshot.datasources.is_empty() {
                        div style="margin-top:1.25rem;padding-top:1rem;border-top:1px solid var(--border)" {
                            span class="mono kpi-label" { "DISCOVERED DATASOURCES" }
                            div class="tag-row" style="margin-top:0.5rem" {
                                @for ds in &snapshot.datasources {
                                    span class="tag" {
                                        (ds.name) " (" (ds.ds_type) ")"
                                        @if ds.is_default { " ★" }
                                    }
                                }
                            }
                        }
                    }
                }

                div class="card" {
                    p class="eyebrow" { "Timeline" }
                    h2 class="card-title" style="margin:0.25rem 0 0.9rem" { "Recent Annotations" }
                    div data-annotations-body="" {
                        @if snapshot.annotations.is_empty() {
                            p class="card-desc" { "No Grafana annotations available." }
                        } @else {
                            ul class="annotation-list" {
                                @for ann in &snapshot.annotations {
                                    li class="annotation-item" {
                                        div class="mono muted-text" style="font-size:0.72rem" {
                                            (format_epoch_ms_short(ann.time_ms))
                                        }
                                        p class="annotation-text" { (ann.text) }
                                        @if !ann.tags.is_empty() {
                                            div class="tag-row" {
                                                @for tag in &ann.tags {
                                                    span class="tag" { (tag) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        section id="setup" class="band" {
            div class="container" {
                details class="setup-details" open=[if show_setup_open { Some("") } else { None }] {
                    summary class="setup-summary" {
                        div {
                            span class="eyebrow" { "Integration & Diagnostics" }
                            h2 class="card-title" style="margin-top:0.25rem" { "Grafana API Key Configuration" }
                        }
                        span class="mono muted-text" style="font-size:0.8rem" { "Toggle instructions & diagnostics" }
                    }

                    div class="setup-body grid-2" {
                        div {
                            h3 class="mono" style="font-size:0.9rem;margin-top:0" { "1. Generate a Grafana Service Account Token" }
                            ol class="setup-steps" {
                                li { "Open Grafana → Administration → Users and access → Service accounts." }
                                li { "Create a Service Account (e.g. urashyl-portfolio) with the Viewer role." }
                                li { "Click Add service account token and copy the glsa_... key." }
                                li { "Copy .env.example to .env in the project root (or set environment variables) and restart the server." }
                            }

                            div class="diagnostics-box" {
                                span class="mono kpi-label" { "LIVE BACKEND DIAGNOSTICS" }
                                ul class="clean-list compact" data-diagnostics-list="" {
                                    @for line in &snapshot.diagnostics {
                                        li class="mono" style="font-size:0.78rem" { (line) }
                                    }
                                }
                            }
                        }

                        div {
                            div class="panel-header" style="margin-bottom:0.5rem" {
                                span class="mono kpi-label" { ".ENV TEMPLATE" }
                                button type="button" id="copy-env-btn" class="btn-ghost btn-sm" { "Copy .env" }
                            }
                            pre id="env-snippet" class="code-block mono" { code { (env_snippet) } }
                            p class="card-desc" style="margin-top:0.75rem" {
                                "Auto-supports Proxmox prometheus-pve-exporter, Linux node_exporter, and Proxmox InfluxDB (set GRAFANA_DATASOURCE_TYPE=influxdb). Override any query via GRAFANA_QUERY_*."
                            }
                        }
                    }
                }
            }
        }
    };

    render_shell(
        "Telemetry — urashyl.dev",
        "Live Proxmox VE & homelab telemetry polled by Rust from Grafana HTTP API.",
        ActivePage::Telemetry,
        snapshot,
        body,
    )
}

fn render_metric_panel(series: &MetricSeries, accent: &str) -> Markup {
    html! {
        article class="metric-card" data-metric-card=(series.key) {
            div class="metric-card-head" {
                div {
                    span class="mono kpi-label" data-metric-short-label=(series.key) {
                        @if series.unit == "guests" {
                            "GUESTS"
                        } @else {
                            (series.key.to_ascii_uppercase())
                        }
                    }
                    h2 class="metric-card-title" data-metric-title=(series.key) { (series.label) }
                }
                span class="mono tabular metric-current" data-metric-value=(series.key) {
                    (format_metric_value(series.current, &series.unit))
                }
            }

            (render_metric_chart(series, accent))

            div class="metric-stats-row mono tabular" {
                span {
                    "min "
                    span data-metric-min=(series.key) { (format_metric_value(series.min, &series.unit)) }
                }
                span {
                    "avg "
                    span data-metric-avg=(series.key) { (format_metric_value(series.avg, &series.unit)) }
                }
                span {
                    "max "
                    span data-metric-max=(series.key) { (format_metric_value(series.max, &series.unit)) }
                }
            }
        }
    }
}

pub fn render_about(snapshot: &TelemetrySnapshot) -> Markup {
    let body = html! {
        section class="page-head-band" {
            div class="container about-hero" {
                div class="about-identity" {
                    img src=(PROFILE.avatar_url) alt=(PROFILE.handle) class="about-avatar" width="84" height="84";
                    div {
                        p class="eyebrow" { "04 // Profile & Architecture" }
                        h1 class="page-title" style="margin-top:0.3rem" { "@" (PROFILE.handle) " — " (PROFILE.domain) }
                        p class="page-lead" style="margin-top:0.65rem" {
                            (PROFILE.headline) " " (PROFILE.subheadline)
                        }
                        div class="hero-cta-row" style="margin-top:1.25rem" {
                            a href=(PROFILE.github_url) target="_blank" rel="noreferrer" class="btn-primary btn-sm" {
                                (github_icon())
                                span { "github.com/urashyl" }
                            }
                            a href="/projects" class="btn-secondary btn-sm" { "Browse Case Studies" }
                            a href="/telemetry" class="btn-ghost btn-sm" { "Live Grafana Telemetry" }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container" {
                p class="eyebrow" { "Engineering Principles" }
                h2 class="section-title" style="margin-bottom:1.25rem" { "How Systems Are Built Here" }

                div class="grid-3" {
                    @for p in PRINCIPLES {
                        div class="card" {
                            span class="mono accent-text" style="font-size:0.8rem;font-weight:600" { (p.index) }
                            h3 class="card-title" style="margin-top:0.5rem" { (p.title) }
                            p class="card-desc" { (p.summary) }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container" {
                p class="eyebrow" { "Tooling & Environment" }
                h2 class="section-title" style="margin-bottom:1.25rem" { "Technical Stack Matrix" }

                div class="grid-4" {
                    @for group in STACK_GROUPS {
                        div class="card" {
                            h3 class="mono stack-group-title" { (group.title) }
                            ul class="stack-list" {
                                @for item in group.items {
                                    li {
                                        span class="stack-dot" {}
                                        span class="mono" { (item) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        section class="band" {
            div class="container grid-2" {
                div class="card" {
                    p class="eyebrow" { "Colophon" }
                    h2 class="card-title" style="margin-top:0.35rem" { "About This Site" }
                    p class="card-desc" {
                        "urashyl.dev is open-source (MIT License) and runs as a single compiled Rust binary using Axum 0.8, Tokio, Maud compile-time HTML templates, and reqwest over rustls-tls. CSS, JavaScript, and SVG icons are embedded directly in the executable with zero Node.js runtime."
                    }
                    p class="card-desc" {
                        "Live telemetry polls Grafana's HTTP API exclusively on a fixed server-side schedule using a read-only Service Account token; public requests and Server-Sent Events (/api/telemetry/stream) read strictly from the sanitized server cache."
                    }
                }

                div class="card" {
                    p class="eyebrow" { "Endpoints & Links" }
                    h2 class="card-title" style="margin-top:0.35rem" { "Public JSON & Stream Routes" }
                    ul class="clean-list compact mono" style="font-size:0.82rem" {
                        li { a href="/api/health" { "GET /api/health" } }
                        li { a href="/api/telemetry" { "GET /api/telemetry" } }
                        li { a href="/api/telemetry/stream" { "GET /api/telemetry/stream (SSE)" } }
                        li { a href="/api/projects" { "GET /api/projects" } }
                        li { a href="/api/services" { "GET /api/services" } }
                        li { a href="/api/search?q=rust" { "GET /api/search?q=…" } }
                    }
                }
            }
        }
    };

    render_shell(
        "About — urashyl.dev",
        "Engineering background, technical stack, and architecture colophon for urashyl.dev.",
        ActivePage::About,
        snapshot,
        body,
    )
}

pub fn render_not_found(path: &str, snapshot: &TelemetrySnapshot) -> Markup {
    let body = html! {
        section class="band" {
            div class="container not-found-box" {
                p class="eyebrow" { "HTTP 404 // Route Not Found" }
                h1 class="page-title" { "Nothing mounted at " code class="mono" { (path) } }
                p class="page-lead" {
                    "The requested route or project slug does not exist on this server."
                }
                div class="hero-cta-row" {
                    a href="/" class="btn-primary btn-sm" { "Back to Overview" }
                    a href="/projects" class="btn-secondary btn-sm" { "All Projects" }
                    a href="/telemetry" class="btn-ghost btn-sm" { "Live Telemetry" }
                }
            }
        }
    };

    render_shell(
        "404 Not Found — urashyl.dev",
        "Page not found on urashyl.dev.",
        ActivePage::NotFound,
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
