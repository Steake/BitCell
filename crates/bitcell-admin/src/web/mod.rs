//! Web interface module

pub mod dashboard;

use tera::Tera;
use std::sync::OnceLock;

static TEMPLATES: OnceLock<Tera> = OnceLock::new();

pub fn templates() -> &'static Tera {
    TEMPLATES.get_or_init(|| {
        match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Template parsing error: {}", e);
                Tera::default()
            }
        }
    })
}
