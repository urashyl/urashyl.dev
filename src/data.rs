use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub handle: &'static str,
    pub domain: &'static str,
    pub github_url: &'static str,
    pub avatar_url: &'static str,
    pub location_note: &'static str,
    pub headline: &'static str,
    pub subheadline: &'static str,
    pub uptime: &'static str,
    pub hypervisor: &'static str,
    pub os: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectCategory {
    Systems,
    Desktop,
}

impl ProjectCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Systems => "systems",
            Self::Desktop => "desktop",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Systems => "Systems",
            Self::Desktop => "Desktop",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "systems" => Some(Self::Systems),
            "desktop" => Some(Self::Desktop),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchLayer {
    pub layer: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: &'static str,
    pub name: &'static str,
    pub tagline: &'static str,
    pub description: &'static str,
    pub overview: &'static str,
    pub highlights: &'static [&'static str],
    pub architecture: &'static [ArchLayer],
    pub decisions: &'static [&'static str],
    pub role: &'static str,
    pub language: &'static str,
    pub tags: &'static [&'static str],
    pub category: ProjectCategory,
    pub featured: bool,
    pub repo_url: &'static str,
    pub live_url: Option<&'static str>,
    pub updated_at: &'static str,
    pub status_note: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceCategory {
    Security,
    Storage,
    Media,
    Infrastructure,
    Machines,
}

impl ServiceCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Storage => "storage",
            Self::Media => "media",
            Self::Infrastructure => "infrastructure",
            Self::Machines => "machines",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Security => "Passwords and secrets",
            Self::Storage => "Files and everyday tools",
            Self::Media => "Photos and media",
            Self::Infrastructure => "Network and monitoring",
            Self::Machines => "Virtual machines",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "security" => Some(Self::Security),
            "storage" => Some(Self::Storage),
            "media" => Some(Self::Media),
            "infrastructure" => Some(Self::Infrastructure),
            "machines" => Some(Self::Machines),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HomelabService {
    pub id: &'static str,
    pub name: &'static str,
    pub role: &'static str,
    pub category: ServiceCategory,
    pub runtime: &'static str,
    pub exposure: &'static str,
    pub probe_keywords: &'static [&'static str],
    pub description: &'static str,
    pub highlights: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct StackGroup {
    pub title: &'static str,
    pub items: &'static [&'static str],
}

pub const PROFILE: Profile = Profile {
    handle: "urashyl",
    domain: "urashyl.dev",
    github_url: "https://github.com/urashyl",
    avatar_url: "https://avatars.githubusercontent.com/u/109770019?v=4",
    location_note: "Arch Linux · Proxmox VE · Self-hosted",
    headline: "Rust backends, hardened Linux systems, and self-hosted infrastructure.",
    subheadline: "Current projects: a multi-tenant Rust web platform, an amnesic Arch Linux live system, hardened sandboxes for AI coding agents, and a niri Wayland desktop.",
    uptime: "99.9%",
    hypervisor: "Proxmox VE",
    os: "Arch Linux (linux-hardened)",
};

pub const PROJECTS: &[Project] = &[
    Project {
        id: "mitch-pro",
        name: "mitch.pro",
        tagline: "Multi-tenant Rust/Axum platform with E2EE chat & Proxmox cloud desktops",
        description: "High-performance self-hosted web platform ported from Bun/Node to a single Rust Axum + Tokio binary with SQLite WAL persistence. Serves multiple domains (mitch.pro, rjuhsd.school, sexypickleclub.com) with shared identity.",
        overview: "Originally prototyped in TypeScript on Bun/Node, mitch.pro outgrew dynamic runtime overhead and fragmented state management. The platform was rewritten into a single statically compiled Rust binary using Axum, Tokio, and SQLite in WAL mode, consolidating multi-domain routing, end-to-end encrypted messaging, and on-demand Proxmox LXC cloud desktops behind one predictable memory footprint.",
        highlights: &[
            "E2EE chat via Conduit Matrix + Cinny & WebCrypto ECDH P-256 / AES-GCM DMs",
            "Ephemeral Proxmox LXC cloud desktops with in-browser VNC/SSH gateway",
            "Argon2id auth, same-origin CSRF protection & real-time WebSockets",
        ],
        architecture: &[
            ArchLayer {
                layer: "Edge & Multi-Tenancy",
                detail: "Single Axum router inspects request Host headers to serve distinct tenant themes and namespaces across mitch.pro, rjuhsd.school, and sexypickleclub.com with shared session identity.",
            },
            ArchLayer {
                layer: "Identity & Cryptography",
                detail: "Argon2id password hashing, hardened cookie sessions with same-origin CSRF enforcement, and client-side WebCrypto ECDH P-256 + AES-GCM key exchange for direct messages alongside federated Conduit Matrix.",
            },
            ArchLayer {
                layer: "Cloud Desktop Provisioner",
                detail: "Talks directly to Proxmox VE APIs to spin up ephemeral LXC desktop sandboxes and proxies interactive VNC/SSH streams cleanly through WebSockets.",
            },
            ArchLayer {
                layer: "Persistence",
                detail: "Embedded SQLite configured with WAL mode, strict foreign keys, and minimal connection contention for single-node durability without external database sprawl.",
            },
        ],
        decisions: &[
            "Replaced Bun/Node service cluster with one compiled Rust binary to cut idle memory usage and eliminate runtime dependency drift.",
            "Selected SQLite WAL over an external Postgres daemon to keep backups atomic via snapshots and latency deterministic.",
            "Kept DM encryption keys purely client-side via WebCrypto so the server acts only as an authenticated relay.",
        ],
        role: "Creator & Systems Architect",
        language: "Rust",
        tags: &["Rust", "Axum", "Tokio", "SQLite", "Matrix", "Proxmox"],
        category: ProjectCategory::Systems,
        featured: true,
        repo_url: "https://github.com/mitch-pro/mitch.pro",
        live_url: Some("https://mitch.pro"),
        updated_at: "Nov 2026",
        status_note: None,
    },
    Project {
        id: "lacuna",
        name: "lacuna",
        tagline: "Amnesic Arch Linux live system with deniable persistence",
        description: "Runs entirely from RAM with zero swap, read-only internal disks, and a default-drop nftables network gate. Routes traffic exclusively through Tor, Mullvad/IVPN/PIA, or Tor-over-VPN, with VeraCrypt/Shufflecake deniable persistence.",
        overview: "lacuna explores building an amnesic live workstation on Arch Linux without sacrificing modern desktop usability or reproducible build automation. Every boot runs inside a RAM-backed root filesystem with swap disabled and host block devices kept read-only unless explicitly unlocked inside a deniable VeraCrypt or Shufflecake container.",
        highlights: &[
            "Default-drop nftables firewall & automated leaktest verification",
            "RAM-only execution; wipe & power-off on USB pull or idle timeout",
            "Separate unprivileged session vs. explicit sudo elevation",
        ],
        architecture: &[
            ArchLayer {
                layer: "Volatile Runtime",
                detail: "Custom Archiso profile boots squashed rootfs into tmpfs/zram, leaving host storage untouched and scrubbing memory on panic, USB removal, or idle watchdog.",
            },
            ArchLayer {
                layer: "Network Gate",
                detail: "Strict default-drop nftables ruleset blocks non-tunnel interfaces at kernel level, permitting egress exclusively via Tor transparent/SOCKS routing, WireGuard VPNs, or Tor-over-VPN chains.",
            },
            ArchLayer {
                layer: "Deniable Storage",
                detail: "Integrates VeraCrypt hidden volumes and Shufflecake plausible-deniability block layers for persistent notes and keys without revealing volume existence.",
            },
        ],
        decisions: &[
            "Built automated leaktest tooling inside the image pipeline to verify DNS, IPv6, and WebRTC isolation before trusting any profile change.",
            "Separated unprivileged user session from explicit administrative elevation so compromised desktop apps cannot alter nftables rules.",
        ],
        role: "Author & Security Engineer",
        language: "Python",
        tags: &["Arch Linux", "nftables", "Tor", "VeraCrypt", "Shufflecake"],
        category: ProjectCategory::Systems,
        featured: true,
        repo_url: "https://github.com/urashyl/lacuna",
        live_url: None,
        updated_at: "Nov 2026",
        status_note: Some("Experimental / Unaudited"),
    },
    Project {
        id: "ai-container",
        name: "ai-container",
        tagline: "Hardened Alpine sandboxes for AI coding agents",
        description: "Runs rootful Podman containers on linux-hardened without enabling unprivileged user namespaces. Maps container root to unprivileged /etc/subuid ranges, mounts a shared 16 GB ext4 root, and isolates networking via pasta.",
        overview: "Running autonomous coding agents locally introduces filesystem and loopback risk, yet enabling unprivileged user namespaces weakens the attack surface reductions of linux-hardened. ai-container solves both by using rootful Podman with explicit subordinate UID/GID isolation and a dedicated loopback-free pasta network stack.",
        highlights: &[
            "Zero unprivileged user namespaces required on linux-hardened",
            "Fixed aibox-run helper with strict uid/cwd sanitization",
            "Isolated pasta network namespace with no host loopback access",
        ],
        architecture: &[
            ArchLayer {
                layer: "Kernel & User Mapping",
                detail: "Preserves linux-hardened sysctl posture (`kernel.unprivileged_userns_clone=0`) while confining container UID 0 to unprivileged host subuid/subgid slices.",
            },
            ArchLayer {
                layer: "Controlled Invocation",
                detail: "Hardened `aibox-run` wrapper validates project working directories and target UIDs before mounting only the designated workspace into an Alpine sandbox.",
            },
            ArchLayer {
                layer: "Network Namespace",
                detail: "Uses `pasta` user-mode networking configured to strip access to host loopback services and LAN ranges unless explicitly bridged.",
            },
        ],
        decisions: &[
            "Backed toolchains on a shared 16 GB ext4 image so ephemeral agent sandboxes start instantly without bloated container layer copies.",
            "Chose Alpine Linux inside the guest to minimize CVE surface area and shell footprint.",
        ],
        role: "Creator & Maintainer",
        language: "Shell",
        tags: &["Podman", "Alpine", "linux-hardened", "pasta", "subuid"],
        category: ProjectCategory::Systems,
        featured: true,
        repo_url: "https://github.com/urashyl/ai-container",
        live_url: None,
        updated_at: "Oct 2026",
        status_note: None,
    },
    Project {
        id: "niri",
        name: "niri",
        tagline: "Wayland compositor rice with Quickshell & dynamic matugen theming",
        description: "Complete Arch Linux desktop environment built around niri and Quickshell. Syncs wallpaper palettes across the bar, kitty, zsh/p10k, and SilentSDDM, paired with custom polkit network-privacy helpers and GRUB/Plymouth boot theming.",
        overview: "A cohesive Wayland workstation engineered around niri's scrollable-tiling model and a bespoke Quickshell interface. Rather than static dotfiles that drift over time, the repository uses idempotent installation and sync scripts that back up existing configuration and propagate matugen color tokens across compositor, terminal, shell, and display manager.",
        highlights: &[
            "Quickshell bar, quick settings, notification center & launcher",
            "Wallpaper-driven matugen color generation across shell & SDDM",
            "Idempotent install.sh & sync.sh with automatic config backups",
        ],
        architecture: &[
            ArchLayer {
                layer: "Shell UI (QML / Quickshell)",
                detail: "Custom bar, notification daemon, app launcher, and quick-settings toggles written directly in QML without Electron or heavy GTK shell wrappers.",
            },
            ArchLayer {
                layer: "Unified Color Pipeline",
                detail: "matugen extracts semantic color roles from the active wallpaper and regenerates theme tokens for Quickshell, kitty, zsh/p10k, and SilentSDDM in one pass.",
            },
            ArchLayer {
                layer: "Boot & Policy Integration",
                detail: "Includes custom GRUB/Plymouth theming and polkit helpers for toggling privacy/network states cleanly from the bar.",
            },
        ],
        decisions: &[
            "Adopted niri scrollable tiling over traditional grid tiling to keep wide terminals and editors readable without manual layout juggling.",
            "Built idempotent sync scripts with timestamped backups so workstation rebuilds remain safe and repeatable.",
        ],
        role: "Author",
        language: "QML",
        tags: &["niri", "Quickshell", "Wayland", "matugen", "Arch"],
        category: ProjectCategory::Desktop,
        featured: true,
        repo_url: "https://github.com/urashyl/niri",
        live_url: None,
        updated_at: "Nov 2026",
        status_note: None,
    },
];

pub const HOMELAB_SERVICES: &[HomelabService] = &[
    HomelabService {
        id: "bitwarden",
        name: "Vaultwarden",
        role: "Password manager",
        category: ServiceCategory::Security,
        runtime: "LXC container",
        exposure: "Public, behind Caddy",
        probe_keywords: &["vaultwarden", "bitwarden"],
        description: "Passwords, passkeys, and TOTP codes, end-to-end encrypted and used through the Bitwarden clients.",
        highlights: &["End-to-end encrypted", "Passkeys and TOTP"],
    },
    HomelabService {
        id: "infisical",
        name: "Infisical",
        role: "Secrets and environment variables",
        category: ServiceCategory::Security,
        runtime: "LXC container",
        exposure: "LAN only",
        probe_keywords: &["infisical"],
        description: "Versioned storage for API keys and tokens, injected at runtime instead of kept in .env files.",
        highlights: &["Versioned secrets", "No plaintext .env files"],
    },
    HomelabService {
        id: "authentik",
        name: "Authentik",
        role: "Single sign-on",
        category: ServiceCategory::Security,
        runtime: "LXC container",
        exposure: "Public, behind Caddy",
        probe_keywords: &["authentik"],
        description: "Single sign-on with 2FA in front of other services.",
        highlights: &["OIDC and forward auth", "WebAuthn"],
    },
    HomelabService {
        id: "crowdsec",
        name: "CrowdSec",
        role: "Intrusion prevention",
        category: ServiceCategory::Security,
        runtime: "LXC container",
        exposure: "Internal only",
        probe_keywords: &["crowdsec"],
        description: "Bans IPs that scan or brute-force the public services, and pulls in blocklists shared by other CrowdSec users.",
        highlights: &["Community blocklists", "Bans at the proxy"],
    },
    HomelabService {
        id: "nextcloud",
        name: "Nextcloud",
        role: "Files, calendar, contacts",
        category: ServiceCategory::Storage,
        runtime: "LXC container on Alpine",
        exposure: "Public, behind Caddy",
        probe_keywords: &["nextcloud"],
        description: "File sync, calendars, and contacts.",
        highlights: &["CalDAV and CardDAV", "Desktop and mobile sync"],
    },
    HomelabService {
        id: "memos",
        name: "Memos",
        role: "Notes and runbooks",
        category: ServiceCategory::Storage,
        runtime: "LXC container with SQLite",
        exposure: "LAN only",
        probe_keywords: &["memos"],
        description: "Markdown notes and runbooks.",
        highlights: &["Markdown", "Instant capture"],
    },
    HomelabService {
        id: "mealie",
        name: "Mealie",
        role: "Recipes and meal planning",
        category: ServiceCategory::Storage,
        runtime: "LXC container",
        exposure: "LAN only",
        probe_keywords: &["mealie"],
        description: "Recipe manager with meal planning and shopping lists.",
        highlights: &["Recipe import from URLs", "Shopping lists"],
    },
    HomelabService {
        id: "ntfy",
        name: "ntfy",
        role: "Push notifications",
        category: ServiceCategory::Storage,
        runtime: "LXC container",
        exposure: "Public, behind Caddy",
        probe_keywords: &["ntfy"],
        description: "Push notifications to phones, sent over plain HTTP.",
        highlights: &["Self-hosted push", "Works from curl"],
    },
    HomelabService {
        id: "immich",
        name: "Immich",
        role: "Photo and video backup",
        category: ServiceCategory::Media,
        runtime: "LXC container",
        exposure: "Public, behind Caddy",
        probe_keywords: &["immich"],
        description: "Photo and video backup from phones, with search powered by locally run machine learning.",
        highlights: &["Local ML search", "Automatic phone backup"],
    },
    HomelabService {
        id: "jellyfin",
        name: "Jellyfin",
        role: "Movies, shows, music",
        category: ServiceCategory::Media,
        runtime: "LXC container with hardware transcoding",
        exposure: "Public, behind Caddy",
        probe_keywords: &["jellyfin"],
        description: "Media server for movies, shows, and music, with hardware transcoding.",
        highlights: &["Hardware transcoding", "No tracking"],
    },
    HomelabService {
        id: "caddy",
        name: "Caddy",
        role: "Reverse proxy",
        category: ServiceCategory::Infrastructure,
        runtime: "LXC container",
        exposure: "The only public entry point",
        probe_keywords: &["caddy", "nginx"],
        description: "Terminates TLS and routes each subdomain to the right container or VM. Nothing else is reachable from outside.",
        highlights: &["TLS for every subdomain", "Single ingress"],
    },
    HomelabService {
        id: "grafana",
        name: "Grafana and Prometheus",
        role: "Metrics and alerts",
        category: ServiceCategory::Infrastructure,
        runtime: "Separate LXC containers",
        exposure: "LAN only",
        probe_keywords: &["grafana", "prom-monitoring", "prometheus"],
        description: "Prometheus scrapes the Proxmox exporter; Grafana draws the dashboards and handles alerting. The status page on this site reads from it.",
        highlights: &["prometheus-pve-exporter", "Unified alerting"],
    },
    HomelabService {
        id: "uptimekuma",
        name: "Uptime Kuma",
        role: "External checks",
        category: ServiceCategory::Infrastructure,
        runtime: "LXC container",
        exposure: "LAN only",
        probe_keywords: &["uptimekuma", "uptime-kuma"],
        description: "Uptime checks and history for each service.",
        highlights: &["HTTP and TCP checks", "Status history"],
    },
    HomelabService {
        id: "glance",
        name: "Glance",
        role: "Home dashboard",
        category: ServiceCategory::Infrastructure,
        runtime: "LXC container",
        exposure: "LAN only",
        probe_keywords: &["glance"],
        description: "Start page with links to everything in the lab.",
        highlights: &["One page for everything"],
    },
    HomelabService {
        id: "vm-urashyl-dev",
        name: "urashyl.dev",
        role: "This website",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 4 GB RAM, 32 GB disk",
        exposure: "",
        probe_keywords: &["urashyl.dev"],
        description: "Runs this website.",
        highlights: &[],
    },
    HomelabService {
        id: "vm-mitch-pro-production",
        name: "mitch-pro-production",
        role: "mitch.pro, production",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 32 GB RAM, 128 GB disk",
        exposure: "",
        probe_keywords: &["mitch-pro-production"],
        description: "The live mitch.pro platform.",
        highlights: &[],
    },
    HomelabService {
        id: "vm-mitch-pro-dev",
        name: "mitch-pro-dev",
        role: "mitch.pro, development",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 16 GB RAM, 128 GB disk",
        exposure: "",
        probe_keywords: &["mitch-pro-dev"],
        description: "Development instance of mitch.pro.",
        highlights: &[],
    },
    HomelabService {
        id: "vm-arch-linux",
        name: "arch-linux",
        role: "Arch Linux",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 48 GB RAM, 96 GB disk",
        exposure: "",
        probe_keywords: &["arch-linux"],
        description: "",
        highlights: &[],
    },
    HomelabService {
        id: "vm-vpn-box",
        name: "vpn-box",
        role: "VPN",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 16 GB RAM, 64 GB disk",
        exposure: "",
        probe_keywords: &["vpn-box"],
        description: "",
        highlights: &[],
    },
    HomelabService {
        id: "vm-crafty-controller",
        name: "crafty-controller",
        role: "Minecraft servers",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 8 GB RAM, 64 GB disk",
        exposure: "",
        probe_keywords: &["crafty-controller"],
        description: "Minecraft servers managed with Crafty Controller.",
        highlights: &[],
    },
    HomelabService {
        id: "vm-brandon-bot",
        name: "brandon-bot",
        role: "",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 4 GB RAM, 32 GB disk",
        exposure: "",
        probe_keywords: &["brandon-bot"],
        description: "",
        highlights: &[],
    },
    HomelabService {
        id: "vm-goguardian",
        name: "goguardian",
        role: "Serves goguardian.mitch.pro",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 2 GB RAM, 32 GB disk",
        exposure: "",
        probe_keywords: &["goguardian"],
        description: "",
        highlights: &[],
    },
    HomelabService {
        id: "vm-decrypt-this",
        name: "decrypt-this",
        role: "GoGuardian research environment",
        category: ServiceCategory::Machines,
        runtime: "KVM guest, 16 GB RAM, 64 GB disk",
        exposure: "",
        probe_keywords: &["decrypt-this"],
        description: "",
        highlights: &[],
    },
];

pub const STACK_GROUPS: &[StackGroup] = &[
    StackGroup {
        title: "Languages",
        items: &["Rust", "Python", "C and C++", "Bash and Zsh", "QML", "SQL (mostly SQLite)"],
    },
    StackGroup {
        title: "Systems",
        items: &["Arch Linux", "linux-hardened", "Proxmox VE", "LXC and KVM", "Podman with pasta", "ZFS"],
    },
    StackGroup {
        title: "Security and networking",
        items: &["nftables", "Tor and obfs4", "WireGuard", "Argon2id, WebCrypto", "Matrix (Conduit)", "VeraCrypt, Shufflecake"],
    },
    StackGroup {
        title: "Desktop and monitoring",
        items: &["niri on Wayland", "dwm on X11", "Quickshell", "Prometheus", "Grafana", "Caddy"],
    },
];

pub fn find_project(slug: &str) -> Option<&'static Project> {
    PROJECTS.iter().find(|p| p.id == slug)
}

pub fn filter_projects(category: Option<ProjectCategory>, query: &str) -> Vec<&'static Project> {
    let q = query.trim().to_ascii_lowercase();
    PROJECTS
        .iter()
        .filter(|p| {
            if category.is_some_and(|cat| p.category != cat) {
                return false;
            }
            if q.is_empty() {
                return true;
            }
            p.name.to_ascii_lowercase().contains(&q)
                || p.tagline.to_ascii_lowercase().contains(&q)
                || p.description.to_ascii_lowercase().contains(&q)
                || p.language.to_ascii_lowercase().contains(&q)
                || p.tags.iter().any(|t| t.to_ascii_lowercase().contains(&q))
        })
        .collect()
}

pub fn adjacent_projects(slug: &str) -> (Option<&'static Project>, Option<&'static Project>) {
    let Some(idx) = PROJECTS.iter().position(|p| p.id == slug) else {
        return (None, None);
    };
    let prev = if idx > 0 { Some(&PROJECTS[idx - 1]) } else { None };
    let next = if idx + 1 < PROJECTS.len() {
        Some(&PROJECTS[idx + 1])
    } else {
        None
    };
    (prev, next)
}

pub fn filter_services(category: Option<ServiceCategory>) -> Vec<&'static HomelabService> {
    HOMELAB_SERVICES
        .iter()
        .filter(|s| category.is_none_or(|c| s.category == c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_and_looks_up_projects_accurately() {
        assert_eq!(PROJECTS.len(), 4);
        assert!(find_project("mitch-pro").is_some());
        assert!(find_project("calc").is_none());
        assert!(find_project("chadwm").is_none());
        assert!(find_project("dotfiles").is_none());
        assert!(find_project("missing").is_none());

        let rust_projects = filter_projects(Some(ProjectCategory::Systems), "axum");
        assert_eq!(rust_projects.len(), 1);
        assert_eq!(rust_projects[0].id, "mitch-pro");

        let (prev, next) = adjacent_projects("mitch-pro");
        assert!(prev.is_none());
        assert_eq!(next.map(|p| p.id), Some("lacuna"));
    }
}
