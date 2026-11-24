pub mod charts;
pub mod layout;
pub mod pages;

pub use pages::{
    render_about, render_home, render_homelab, render_not_found, render_project_detail,
    render_projects, render_telemetry,
};
