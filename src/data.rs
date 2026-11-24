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
            Self::Systems => "Systems & Rust",
            Self::Desktop => "Desktop & Config",
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
}

impl ServiceCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Storage => "storage",
            Self::Media => "media",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Security => "Secrets & Auth",
            Self::Storage => "Cloud & Notes",
            Self::Media => "Media & Photos",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "security" => Some(Self::Security),
            "storage" => Some(Self::Storage),
            "media" => Some(Self::Media),
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

#[derive(Debug, Clone, Serialize)]
pub struct EngineeringPrinciple {
    pub index: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
}

pub const PROFILE: Profile = Profile {
    handle: "urashyl",
    domain: "urashyl.dev",
    github_url: "https://github.com/urashyl",
    avatar_url: "https://avatars.githubusercontent.com/u/109770019?v=4",
    location_note: "Arch Linux · Proxmox VE · Self-hosted",
    headline: "Rust backends, hardened Linux systems & self-hosted infrastructure.",
    subheadline: "Building multi-tenant Axum platforms, amnesic Arch live systems, hardened Podman sandboxes, and Wayland desktops — backed by a Proxmox VE homelab observed through Grafana.",
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
        id: "infisical",
        name: "Infisical",
        role: "Secrets & Env Management",
        category: ServiceCategory::Security,
        runtime: "Proxmox LXC / Docker",
        exposure: "Private LAN / TLS",
        probe_keywords: &["infisical"],
        description: "Centralized secrets vault for API keys, infrastructure tokens, and environment injection across self-hosted stacks.",
        highlights: &["Versioned secrets", "Zero plaintext .env sprawl"],
    },
    HomelabService {
        id: "bitwarden",
        name: "Bitwarden",
        role: "Zero-Knowledge Password Vault",
        category: ServiceCategory::Security,
        runtime: "Proxmox LXC / TLS",
        exposure: "Reverse Proxy TLS",
        probe_keywords: &["bitwarden", "vaultwarden"],
        description: "Self-hosted credential and passkey vault providing end-to-end encrypted password management across all devices.",
        highlights: &["E2E encrypted", "Passkeys & TOTP"],
    },
    HomelabService {
        id: "immich",
        name: "Immich",
        role: "Photo & Video Backup",
        category: ServiceCategory::Media,
        runtime: "Proxmox VM / GPU",
        exposure: "Reverse Proxy TLS",
        probe_keywords: &["immich"],
        description: "High-performance self-hosted photo and video library with automatic mobile backup and local ML-powered search.",
        highlights: &["Local ML indexing", "Automated mobile sync"],
    },
    HomelabService {
        id: "nextcloud",
        name: "Nextcloud",
        role: "Private Cloud & Sync",
        category: ServiceCategory::Storage,
        runtime: "Proxmox LXC / ZFS",
        exposure: "Reverse Proxy TLS",
        probe_keywords: &["nextcloud"],
        description: "Self-owned file sync, document storage, calendar, and contacts replacing third-party cloud drives.",
        highlights: &["ZFS-backed storage", "CalDAV / CardDAV"],
    },
    HomelabService {
        id: "jellyfin",
        name: "Jellyfin",
        role: "Media Streaming Server",
        category: ServiceCategory::Media,
        runtime: "Proxmox LXC / HW Transcode",
        exposure: "Reverse Proxy TLS",
        probe_keywords: &["jellyfin"],
        description: "Open-source media server delivering movies, shows, and music with hardware-accelerated transcoding and zero telemetry.",
        highlights: &["Hardware transcoding", "Zero tracking"],
    },
    HomelabService {
        id: "memos",
        name: "Memos",
        role: "Knowledge Base & Notes",
        category: ServiceCategory::Storage,
        runtime: "Proxmox LXC / SQLite",
        exposure: "Private LAN / TLS",
        probe_keywords: &["memos"],
        description: "Lightweight, markdown-first self-hosted scratchpad for technical notes, runbooks, and quick daily capture.",
        highlights: &["Markdown-native", "Instant capture"],
    },
];

pub const STACK_GROUPS: &[StackGroup] = &[
    StackGroup {
        title: "Backend & Systems",
        items: &["Rust (Axum / Tokio)", "SQLite (WAL)", "Python", "C / C++", "Bash / Zsh", "QML (Quickshell)"],
    },
    StackGroup {
        title: "OS & Virtualization",
        items: &["Arch Linux", "linux-hardened", "Proxmox VE", "LXC & KVM", "Podman / pasta", "99.9% Uptime"],
    },
    StackGroup {
        title: "Security & Networking",
        items: &["nftables", "Tor & obfs4", "Argon2id / WebCrypto", "Conduit Matrix", "VeraCrypt / Shufflecake", "Caddy / TLS"],
    },
    StackGroup {
        title: "Desktop & Observability",
        items: &["niri (Wayland)", "dwm (X11)", "Grafana + Prometheus", "Infisical & Bitwarden", "Immich & Jellyfin", "Nextcloud & Memos"],
    },
];

pub const PRINCIPLES: &[EngineeringPrinciple] = &[
    EngineeringPrinciple {
        index: "01",
        title: "Single-Binary Simplicity",
        summary: "Favor compiled Rust services with embedded assets and WAL-backed SQLite or static state over sprawling Node/container orchestration when operating lean self-hosted infrastructure.",
    },
    EngineeringPrinciple {
        index: "02",
        title: "Isolation by Default",
        summary: "Whether locking egress behind default-drop nftables in lacuna, confining coding agents in pasta-isolated Podman sandboxes, or splitting homelab workloads into Proxmox LXC guests, trust boundaries come first.",
    },
    EngineeringPrinciple {
        index: "03",
        title: "Own Your Telemetry & Cloud",
        summary: "Passwords, photos, notes, media, and observability live on self-owned hardware behind TLS and Grafana API telemetry — never rented black boxes.",
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
            if let Some(cat) = category
                && p.category != cat
            {
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
