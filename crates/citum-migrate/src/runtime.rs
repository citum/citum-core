/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Small CLI runtime helpers that keep `main.rs` focused on orchestration.

use crate::cli::Args;
use citum_migrate::{
    debug_output::DebugOutputFormatter, provenance::ProvenanceTracker, template_resolver,
};
use citum_schema::Style;
use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
};

/// Extract the style name from the CSL path and resolve template candidates.
pub(crate) fn resolve_style_name_and_templates(
    path: &str,
    cli: &Args,
) -> (String, template_resolver::ResolvedTemplates) {
    let style_name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let workspace_root = workspace_root_for_style_path(path);

    let resolved = template_resolver::resolve_templates(
        path,
        style_name.as_str(),
        cli.template_dir.as_deref(),
        &workspace_root,
        cli.template_mode,
        cli.min_template_confidence,
        cli.live_infer_backend,
    );

    (style_name, resolved)
}

/// Resolve the workspace root that owns the provided style path.
pub(crate) fn workspace_root_for_style_path(path: &str) -> PathBuf {
    let style_path = Path::new(path);
    let rooted_style_path = if style_path.is_absolute() {
        style_path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(style_path)
    };

    let workspace_root = rooted_style_path
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists())
        .unwrap_or(rooted_style_path.parent().unwrap_or(Path::new(".")))
        .to_path_buf();
    fs::canonicalize(&workspace_root).unwrap_or(workspace_root)
}

/// Write the migrated style and optional variable-debug side channel.
pub(crate) fn output_style_and_debug(
    style: &Style,
    debug_variable: Option<&str>,
    tracker: &ProvenanceTracker,
) -> Result<(), Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(style)?;
    writeln!(std::io::stdout(), "{yaml}")?;

    if let Some(var_name) = debug_variable {
        let debug_output = DebugOutputFormatter::format_variable(tracker, var_name);
        eprint!("{debug_output}");
    }

    Ok(())
}

/// Log which template source won each migrated section.
#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
pub(crate) fn log_template_sources(resolved: &template_resolver::ResolvedTemplates) {
    if let Some(ref resolved_bib) = resolved.bibliography {
        tracing::debug!("Using {} bibliography template", resolved_bib.source);
        if let Some(conf) = resolved_bib.confidence {
            tracing::debug!("  bibliography confidence: {:.0}%", conf * 100.0);
        }
    } else {
        tracing::debug!(
            "Using {} bibliography template",
            template_resolver::TemplateSource::XmlCompiled
        );
    }

    if let Some(ref resolved_cit) = resolved.citation {
        tracing::debug!("Using {} citation template", resolved_cit.source);
        if let Some(conf) = resolved_cit.confidence {
            tracing::debug!("  citation confidence: {:.0}%", conf * 100.0);
        }
    } else {
        tracing::debug!(
            "Using {} citation template",
            template_resolver::TemplateSource::XmlCompiled
        );
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "Panicking is acceptable in tests.")]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate should live under crates/citum-migrate")
            .to_path_buf()
    }

    fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("cwd lock should not be poisoned")
    }

    #[test]
    fn workspace_root_resolves_relative_paths_from_subdirectories() {
        let _guard = cwd_lock();
        let original_cwd = std::env::current_dir().expect("current dir should be available");
        std::env::set_current_dir(repo_root().join("crates"))
            .expect("test should enter repo subdirectory");

        let workspace_root = workspace_root_for_style_path("../styles-legacy/apa-6th-edition.csl");

        std::env::set_current_dir(original_cwd).expect("test should restore cwd");
        assert_eq!(workspace_root, repo_root());
    }
}
