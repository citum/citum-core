/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! CLI argument parsing for citum-migrate.

use citum_migrate::template_resolver;
use std::path::PathBuf;

pub(crate) struct CliArgs {
    pub(crate) path: String,
    pub(crate) debug_variable: Option<String>,
    pub(crate) template_mode: template_resolver::TemplateMode,
    pub(crate) live_infer_backend: template_resolver::LiveInferBackend,
    pub(crate) template_dir: Option<PathBuf>,
    pub(crate) min_template_confidence: f64,
    /// Optional sidecar path for the machine-readable migration evidence
    /// record. When set, the file is written alongside the migrated YAML.
    /// Default off keeps the CLI surface deterministic for existing callers.
    pub(crate) emit_evidence: Option<PathBuf>,
    /// Family-candidate routing mode for styles where no parent link is
    /// available in the source CSL. `Off` (default) preserves standalone
    /// output; `Auto` promotes any reverse-template-link the lineage resolver
    /// discovered; `Explicit(id)` forces the given canonical id.
    pub(crate) family_candidate: FamilyCandidateMode,
    /// When set alongside a promoted family-candidate parent, emit a minimal
    /// wrapper form (info + extends only) instead of the full diff. Default
    /// off preserves full diff behavior; used by the evidence-driven
    /// compression harness to derive output-driven minima.
    pub(crate) minimize_wrapper: bool,
}

/// CLI selection mode for `--family-candidate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FamilyCandidateMode {
    /// Standalone migration only; do not promote any discovered candidate.
    Off,
    /// Promote whatever the lineage resolver discovered via reverse template
    /// link scan.
    Auto,
    /// Force the supplied canonical id as the family-candidate parent.
    Explicit(String),
}

fn parse_template_mode_arg(value: &str) -> template_resolver::TemplateMode {
    match value.parse::<template_resolver::TemplateMode>() {
        Ok(mode) => mode,
        Err(msg) => {
            tracing::debug!("Error: {msg}");
            std::process::exit(1);
        }
    }
}

fn parse_live_infer_backend_arg(value: &str) -> template_resolver::LiveInferBackend {
    match value.parse::<template_resolver::LiveInferBackend>() {
        Ok(backend) => backend,
        Err(msg) => {
            tracing::debug!("Error: {msg}");
            std::process::exit(1);
        }
    }
}

fn parse_path_arg(next: Option<&String>, flag: &str) -> PathBuf {
    let Some(val) = next else {
        tracing::debug!("Error: {flag} requires a path argument");
        std::process::exit(1);
    };
    PathBuf::from(val)
}

fn parse_family_candidate_arg(next: Option<&String>) -> FamilyCandidateMode {
    let Some(val) = next else {
        tracing::debug!("Error: --family-candidate requires an argument (off|auto|<style-id>)");
        std::process::exit(1);
    };
    match val.as_str() {
        "off" => FamilyCandidateMode::Off,
        "auto" => FamilyCandidateMode::Auto,
        explicit => FamilyCandidateMode::Explicit(explicit.to_string()),
    }
}

fn parse_min_template_confidence_arg(value: &str) -> f64 {
    match value.parse::<f64>() {
        Ok(parsed) if (0.0..=1.0).contains(&parsed) => parsed,
        _ => {
            tracing::debug!("Error: --min-template-confidence requires a number in [0.0, 1.0]");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
pub(crate) fn parse_cli_args(args: &[String]) -> CliArgs {
    let program_name = args
        .first()
        .and_then(|arg| std::path::Path::new(arg).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("citum-migrate");

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help(program_name);
        std::process::exit(0);
    }

    let mut path = "styles-legacy/apa.csl".to_string();
    let mut debug_variable: Option<String> = None;
    let mut template_mode = template_resolver::TemplateMode::Auto;
    let mut live_infer_backend = template_resolver::LiveInferBackend::Auto;
    let mut template_dir: Option<PathBuf> = None;
    let mut min_template_confidence = 0.70_f64;
    let mut emit_evidence: Option<PathBuf> = None;
    let mut family_candidate = FamilyCandidateMode::Off;
    let mut minimize_wrapper = false;

    let mut iter = args.iter().skip(1).peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--debug-variable" => {
                if let Some(val) = iter.next() {
                    debug_variable = Some(val.clone());
                } else {
                    tracing::debug!("Error: --debug-variable requires an argument");
                    std::process::exit(1);
                }
            }
            "--template-source" => {
                if let Some(val) = iter.next() {
                    template_mode = parse_template_mode_arg(val);
                } else {
                    tracing::debug!(
                        "Error: --template-source requires an argument (auto|hand|inferred|xml)"
                    );
                    std::process::exit(1);
                }
            }
            "--live-infer-backend" => {
                if let Some(val) = iter.next() {
                    live_infer_backend = parse_live_infer_backend_arg(val);
                } else {
                    tracing::debug!(
                        "Error: --live-infer-backend requires an argument (auto|embedded|node)"
                    );
                    std::process::exit(1);
                }
            }
            "--min-template-confidence" => {
                if let Some(val) = iter.next() {
                    min_template_confidence = parse_min_template_confidence_arg(val);
                } else {
                    tracing::debug!("Error: --min-template-confidence requires a numeric argument");
                    std::process::exit(1);
                }
            }
            "--template-dir" => {
                if let Some(val) = iter.next() {
                    template_dir = Some(PathBuf::from(val));
                } else {
                    tracing::debug!("Error: --template-dir requires a path argument");
                    std::process::exit(1);
                }
            }
            "--emit-evidence" => emit_evidence = Some(parse_path_arg(iter.next(), arg)),
            "--family-candidate" => {
                family_candidate = parse_family_candidate_arg(iter.next());
            }
            "--minimize-wrapper" => {
                minimize_wrapper = true;
            }
            arg if !arg.starts_with('-') => {
                path = arg.to_string();
            }
            _ => {
                tracing::debug!("Error: unknown argument '{}'", arg);
                tracing::debug!("");
                print_help(program_name);
                std::process::exit(1);
            }
        }
    }

    CliArgs {
        path,
        debug_variable,
        template_mode,
        live_infer_backend,
        template_dir,
        min_template_confidence,
        emit_evidence,
        family_candidate,
        minimize_wrapper,
    }
}

#[allow(clippy::cognitive_complexity, reason = "macro-heavy output code")]
pub(super) fn print_help(program_name: &str) {
    tracing::debug!("Citum style migration tool");
    tracing::debug!("");
    tracing::debug!("Usage:");
    tracing::debug!("  {program_name} [STYLE.csl] [options]");
    tracing::debug!("");
    tracing::debug!("Arguments:");
    tracing::debug!("  STYLE.csl                       Input CSL 1.0 style path");
    tracing::debug!("                                  (default: styles-legacy/apa.csl)");
    tracing::debug!("");
    tracing::debug!("Options:");
    tracing::debug!("  -h, --help                      Show this help text");
    tracing::debug!("  --debug-variable <name>         Print provenance details for one variable");
    tracing::debug!("  --template-source <mode>        Template source: auto|hand|inferred|xml");
    tracing::debug!("  --live-infer-backend <mode>     Live inference backend: auto|embedded|node");
    tracing::debug!(
        "  --template-dir <path>           Override directory for hand-authored templates"
    );
    tracing::debug!("  --min-template-confidence <n>   Minimum inferred confidence [0.0, 1.0]");
    tracing::debug!(
        "  --emit-evidence <path>          Write machine-readable migration evidence JSON to <path>"
    );
    tracing::debug!(
        "  --family-candidate <mode>       Route through discovered family parent: off|auto|<id>"
    );
    tracing::debug!(
        "  --minimize-wrapper              Emit minimal wrapper (info + extends only) when promoting family-candidate"
    );
}
