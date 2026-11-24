use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{
    asset_version,
    data::PROFILE,
    grafana::{TelemetrySnapshot, TelemetryState},
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

pub struct PageMeta<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub path: &'a str,
    pub active: ActivePage,
}

pub fn state_class(state: TelemetryState) -> &'static str {
    match state {
        TelemetryState::Connected => "state-live",
        TelemetryState::Degraded => "state-degraded",
        TelemetryState::Error => "state-error",
        TelemetryState::Unconfigured => "state-standby",
    }
}

pub fn github_icon() -> PreEscaped<&'static str> {
    PreEscaped(
        r#"<svg class="icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg>"#,
    )
}

pub fn external_icon() -> PreEscaped<&'static str> {
    PreEscaped(
        r#"<svg class="icon icon-ext" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.25" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M7 17 17 7"/><path d="M8 7h9v9"/></svg>"#,
    )
}

fn search_icon() -> PreEscaped<&'static str> {
    PreEscaped(
        r#"<svg class="icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="11" cy="11" r="6.5"/><path d="m16 16 4.5 4.5"/></svg>"#,
    )
}

fn theme_icon() -> PreEscaped<&'static str> {
    PreEscaped(
        r#"<svg class="icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="8"/><path d="M12 4a8 8 0 0 1 0 16Z" fill="currentColor" stroke="none"/></svg>"#,
    )
}

const THEME_BOOT: &str = r#"(()=>{try{const t=localStorage.getItem('urashyl-theme');if(t==='light'||t==='dark')document.documentElement.dataset.theme=t}catch{}})();"#;

pub fn render_shell(meta: PageMeta<'_>, snapshot: &TelemetrySnapshot, body: Markup) -> Markup {
    let nav_items = [
        ("Work", "/projects", ActivePage::Projects),
        ("Homelab", "/homelab", ActivePage::Homelab),
        ("Status", "/telemetry", ActivePage::Telemetry),
        ("About", "/about", ActivePage::About),
    ];
    let canonical = format!("https://{}{}", PROFILE.domain, meta.path);
    let v = asset_version();

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (meta.title) }
                meta name="description" content=(meta.description);
                link rel="canonical" href=(canonical);
                meta property="og:type" content="website";
                meta property="og:site_name" content=(PROFILE.domain);
                meta property="og:title" content=(meta.title);
                meta property="og:description" content=(meta.description);
                meta property="og:url" content=(canonical);
                meta property="og:image" content=(PROFILE.avatar_url);
                meta name="theme-color" media="(prefers-color-scheme: light)" content="#eef0eb";
                meta name="theme-color" media="(prefers-color-scheme: dark)" content="#141916";
                link rel="icon" type="image/svg+xml" href=(format!("/favicon.svg?v={v}"));
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Schibsted+Grotesk:ital,wght@0,400..800;1,400..800&family=JetBrains+Mono:wght@400;500&display=swap";
                script { (PreEscaped(THEME_BOOT)) }
                link rel="stylesheet" href=(format!("/styles.css?v={v}"));
            }
            body {
                a href="#main" class="skip-link" { "Skip to content" }

                header class="site-header" {
                    div class="wrap header-inner" {
                        a href="/" class="site-name" aria-current=[(meta.active == ActivePage::Home).then_some("page")] {
                            img src=(PROFILE.avatar_url) alt="" width="28" height="28";
                            span { (PROFILE.handle) }
                        }
                        nav class="site-nav" aria-label="Main" {
                            @for (label, href, page) in nav_items {
                                a href=(href) aria-current=[(meta.active == page).then_some("page")] {
                                    @if page == ActivePage::Telemetry && snapshot.configured {
                                        span
                                            class=(format!("state-dot {}", state_class(snapshot.state)))
                                            data-state-dot=""
                                            title=(format!("Server: {}", snapshot.state.label()))
                                            {}
                                    }
                                    (label)
                                }
                            }
                        }
                        div class="header-tools" {
                            button type="button" id="open-cmd-palette" class="icon-btn" aria-label="Search the site" title="Search (press /)" {
                                (search_icon())
                            }
                            button type="button" id="theme-toggle" class="icon-btn" aria-label="Switch color theme" title="Switch color theme" {
                                (theme_icon())
                            }
                        }
                    }
                }

                main id="main" { (body) }

                footer class="site-footer" {
                    div class="wrap footer-inner" {
                        p {
                            "Written in Rust and served from my homelab. "
                            a href="https://github.com/urashyl/urashyl.dev" rel="noopener noreferrer" { "Source on GitHub" }
                            "."
                        }
                        p class="footer-links" {
                            a href=(PROFILE.github_url) rel="noopener noreferrer" {
                                (github_icon()) " @" (PROFILE.handle)
                            }
                        }
                    }
                }

                div id="cmd-palette" class="cmd-backdrop hidden" role="dialog" aria-modal="true" aria-label="Search" {
                    div class="cmd-dialog" {
                        div class="cmd-input-row" {
                            (search_icon())
                            input
                                id="cmd-input"
                                type="text"
                                class="cmd-input"
                                placeholder="Search projects, services, pages"
                                autocomplete="off"
                                spellcheck="false"
                                aria-label="Search";
                            button type="button" id="cmd-close" class="cmd-esc" { "Esc" }
                        }
                        div id="cmd-results" class="cmd-results" role="listbox" {}
                    }
                }

                script src=(format!("/app.js?v={v}")) defer="" {}
            }
        }
    }
}
