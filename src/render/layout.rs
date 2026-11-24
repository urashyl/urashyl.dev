use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{
    data::PROFILE,
    grafana::{TelemetrySnapshot, TelemetryState},
    render::charts::format_iso_short,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePage {
    Home,
    Projects,
    Homelab,
    Telemetry,
    About,
    NotFound,
}

pub fn state_class(state: TelemetryState) -> &'static str {
    match state {
        TelemetryState::Connected => "state-live",
        TelemetryState::Degraded => "state-degraded",
        TelemetryState::Error => "state-error",
        TelemetryState::Unconfigured => "state-standby",
    }
}

pub fn state_slug(state: TelemetryState) -> &'static str {
    match state {
        TelemetryState::Connected => "connected",
        TelemetryState::Degraded => "degraded",
        TelemetryState::Error => "error",
        TelemetryState::Unconfigured => "unconfigured",
    }
}

pub fn github_icon() -> PreEscaped<&'static str> {
    PreEscaped(
        r#"<svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg>"#,
    )
}

pub fn arrow_up_right_icon() -> PreEscaped<&'static str> {
    PreEscaped(
        r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M7 17 17 7"/><path d="M7 7h10v10"/></svg>"#,
    )
}

pub fn render_shell(
    title: &str,
    description: &str,
    active: ActivePage,
    snapshot: &TelemetrySnapshot,
    body: Markup,
) -> Markup {
    let pill_text = match (snapshot.state, snapshot.metrics.cpu.current) {
        (TelemetryState::Connected, Some(cpu)) => format!("Live · {cpu:.0}% CPU"),
        _ => format!("Grafana: {}", snapshot.state.label()),
    };

    let nav_items = [
        ("Overview", "/", ActivePage::Home),
        ("Projects", "/projects", ActivePage::Projects),
        ("Homelab", "/homelab", ActivePage::Homelab),
        ("Telemetry", "/telemetry", ActivePage::Telemetry),
        ("About", "/about", ActivePage::About),
    ];

    html! {
        (DOCTYPE)
        html lang="en" data-theme="dark" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                meta name="description" content=(description);
                link rel="icon" type="image/svg+xml" href="/favicon.svg";
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
                link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,400;0,500;0,600;1,400&family=Instrument+Sans:wght@400;500;600;700&family=Instrument+Serif:ital@0;1&display=swap" rel="stylesheet";
                script {
                    (PreEscaped(r#"(() => { try { const t = localStorage.getItem('urashyl-theme'); if (t === 'light' || t === 'dark') { document.documentElement.dataset.theme = t; } } catch {} })();"#))
                }
                link rel="stylesheet" href="/styles.css";
            }
            body {
                a href="#main-content" class="skip-link" { "Skip to content" }

                header class="site-header" {
                    div class="container header-inner" {
                        div class="header-left" {
                            a href="/" class="brand-mark" {
                                span class="brand-glyph" aria-hidden="true" { "~/" }
                                span class="brand-name" { (PROFILE.domain) }
                            }
                            nav class="primary-nav" aria-label="Primary navigation" {
                                @for (label, href, page) in nav_items {
                                    @let is_active = active == page;
                                    a
                                        href=(href)
                                        class=(if is_active { "nav-item active" } else { "nav-item" })
                                        aria-current=[if is_active { Some("page") } else { None }]
                                    {
                                        (label)
                                    }
                                }
                            }
                        }

                        div class="header-right" {
                            a
                                href="/telemetry"
                                class=(format!("telemetry-pill {}", state_class(snapshot.state)))
                                data-telemetry-pill=""
                                data-telemetry-state=(state_slug(snapshot.state))
                                title="View live Grafana telemetry console"
                            {
                                span class="status-dot" {}
                                span data-telemetry-pill-text="" { (pill_text) }
                            }

                            button
                                type="button"
                                id="open-cmd-palette"
                                class="header-icon-btn"
                                title="Search projects, services & telemetry (⌘K)"
                                aria-label="Open command palette"
                            {
                                span class="mono" { "Search" }
                                kbd class="kbd" { "⌘K" }
                            }

                            button
                                type="button"
                                id="theme-toggle"
                                class="header-icon-btn"
                                title="Toggle dark/light theme"
                                aria-label="Toggle color theme"
                            {
                                span id="theme-label" class="mono" { "Light" }
                            }

                            button
                                type="button"
                                id="mobile-nav-toggle"
                                class="header-icon-btn mobile-only"
                                aria-expanded="false"
                                aria-controls="mobile-nav"
                                aria-label="Toggle navigation menu"
                            {
                                span class="mono" { "Menu" }
                            }
                        }
                    }

                    nav id="mobile-nav" class="mobile-nav hidden" aria-label="Mobile navigation" {
                        div class="container mobile-nav-inner" {
                            @for (label, href, page) in nav_items {
                                @let is_active = active == page;
                                a
                                    href=(href)
                                    class=(if is_active { "mobile-nav-item active" } else { "mobile-nav-item" })
                                    aria-current=[if is_active { Some("page") } else { None }]
                                {
                                    (label)
                                }
                            }
                        }
                    }
                }

                main id="main-content" class="page-main" {
                    (body)
                }

                footer class="site-footer" {
                    div class="container footer-grid" {
                        div class="footer-col-main" {
                            div class="footer-brand" {
                                span class="mono" style="font-weight:600" { (PROFILE.domain) }
                                span class="sep" { "·" }
                                span class=(format!("mono footer-state {}", state_class(snapshot.state))) data-footer-state="" {
                                    "Grafana: " (snapshot.state.label())
                                }
                            }
                            p class="footer-meta" {
                                "Open-source Rust (Axum + Maud) · MIT License · " (PROFILE.os) " · " (PROFILE.hypervisor)
                            }
                            p class="footer-meta" {
                                "Last Grafana sync: "
                                span class="mono tabular" data-last-sync="" {
                                    (format_iso_short(snapshot.last_synced_at.as_deref()))
                                }
                            }
                        }

                        div class="footer-links" {
                            a href="/" { "Overview" }
                            a href="/projects" { "Projects" }
                            a href="/homelab" { "Homelab" }
                            a href="/telemetry" { "Telemetry" }
                            a href="/about" { "About" }
                        }

                        div class="footer-actions" {
                            button type="button" id="copy-domain-btn" class="btn-ghost btn-sm" {
                                span id="copy-domain-label" { "Copy " (PROFILE.domain) }
                            }
                            a href=(PROFILE.github_url) target="_blank" rel="noreferrer" class="btn-ghost btn-sm" {
                                (github_icon())
                                span { "@" (PROFILE.handle) }
                            }
                        }
                    }
                }

                // Command Palette Modal
                div id="cmd-palette" class="cmd-backdrop hidden" role="dialog" aria-modal="true" aria-label="Search & navigation palette" {
                    div class="cmd-dialog" {
                        div class="cmd-input-row" {
                            span class="mono muted-text" { "/" }
                            input
                                id="cmd-input"
                                type="text"
                                class="cmd-input"
                                placeholder="Jump to projects, homelab services, Grafana dashboards..."
                                autocomplete="off"
                                spellcheck="false"
                                aria-label="Search query";
                            button type="button" id="cmd-close" class="kbd-btn" { "ESC" }
                        }
                        div id="cmd-results" class="cmd-results" {}
                        div class="cmd-foot" {
                            span class="mono muted-text" { "↑↓ navigate · ↵ open · esc close" }
                            span class="mono muted-text" { "/api/search" }
                        }
                    }
                }

                script src="/app.js" defer="" {}
            }
        }
    }
}
