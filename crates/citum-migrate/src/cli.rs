/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! CLI argument parsing for citum-migrate.

use citum_migrate::template_resolver;
use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use std::path::PathBuf;

const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

/// Migrate a CSL 1.0 style (.csl) to a Citum style (.yaml).
///
/// Output is written to stdout; redirect to a file as needed.
#[derive(Parser, Debug)]
#[command(name = "citum-migrate", version, styles = CLAP_STYLES)]
pub(crate) struct Args {
    /// Input CSL 1.0 style path
    #[arg(default_value = "styles-legacy/apa.csl", value_name = "STYLE.csl")]
    pub(crate) path: String,

    /// Print provenance details for one variable
    #[arg(long, value_name = "NAME")]
    pub(crate) debug_variable: Option<String>,

    /// Template source strategy
    ///
    /// auto: hand-authored → inferred cache/live → XML fallback
    /// hand: hand-authored only → XML fallback
    /// inferred: inferred cache only, never runs live inference
    /// xml: XML template compiler only
    #[arg(
        long = "template-source",
        value_name = "MODE",
        default_value = "auto",
        value_parser = parse_template_mode
    )]
    pub(crate) template_mode: template_resolver::TemplateMode,

    /// Live inference backend
    ///
    /// auto: embedded JS first, then Node subprocess fallback
    /// embedded: embedded JS runtime only
    /// node: legacy Node subprocess only
    ///
    /// Only applies when --template-source auto needs live inference after
    /// cache lookup. Cache hits win first.
    #[arg(
        long = "live-infer-backend",
        value_name = "MODE",
        default_value = "auto",
        value_parser = parse_live_infer_backend
    )]
    pub(crate) live_infer_backend: template_resolver::LiveInferBackend,

    /// Override directory for hand-authored templates
    #[arg(long, value_name = "PATH")]
    pub(crate) template_dir: Option<PathBuf>,

    /// Minimum inferred template confidence gate [0.0, 1.0]
    ///
    /// Rejects inferred fragments below this threshold; the migration then
    /// falls back to XML template compilation for that section.
    #[arg(long, value_name = "N", default_value = "0.70", value_parser = parse_confidence)]
    pub(crate) min_template_confidence: f64,

    /// Write machine-readable migration evidence JSON to PATH
    #[arg(long, value_name = "PATH")]
    pub(crate) emit_evidence: Option<PathBuf>,

    /// Family-candidate routing: off | auto | <style-id>
    ///
    /// Omitted (default): preserve standalone output.
    /// off: explicitly disable promotion.
    /// auto: promote whatever the lineage resolver discovered via reverse
    ///       template-link scan.
    /// <style-id>: force the given canonical ID as the family-candidate parent.
    #[arg(long, value_name = "MODE")]
    pub(crate) family_candidate: Option<String>,

    /// Emit minimal wrapper (info + extends only) when promoting a family candidate
    #[arg(long)]
    pub(crate) minimize_wrapper: bool,
}

/// CLI selection mode for `--family-candidate`, resolved from the raw string arg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FamilyCandidateMode {
    /// Default no-op routing; preserve standalone output.
    Default,
    /// Standalone migration only; do not promote any discovered candidate.
    Off,
    /// Promote whatever the lineage resolver discovered via reverse template link scan.
    Auto,
    /// Force the supplied canonical id as the family-candidate parent.
    Explicit(String),
}

impl FamilyCandidateMode {
    pub(crate) fn from_arg(raw: Option<&str>) -> Self {
        match raw {
            None => Self::Default,
            Some("off") => Self::Off,
            Some("auto") => Self::Auto,
            Some(id) => Self::Explicit(id.to_string()),
        }
    }
}

fn parse_confidence(s: &str) -> Result<f64, String> {
    let v: f64 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    if (0.0..=1.0).contains(&v) {
        Ok(v)
    } else {
        Err(format!("'{v}' is out of range; must be in [0.0, 1.0]"))
    }
}

fn parse_template_mode(s: &str) -> Result<template_resolver::TemplateMode, String> {
    s.parse::<template_resolver::TemplateMode>()
}

fn parse_live_infer_backend(s: &str) -> Result<template_resolver::LiveInferBackend, String> {
    s.parse::<template_resolver::LiveInferBackend>()
}
