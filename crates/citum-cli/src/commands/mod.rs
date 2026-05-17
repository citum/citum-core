/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! CLI subcommand dispatch and shared aliases.

#[cfg(feature = "typescript")]
mod bindings;
mod catalog;
mod check;
mod convert;
mod doctor;
mod lint;
mod locale;
mod registry;
mod render;
#[cfg(feature = "schema")]
mod schema;
mod style;
mod style_install;
mod util;

pub(crate) use catalog::StyleCatalogRow;

use crate::args::{CheckArgs, Cli, Commands, InputFormat, RefsFormat, RenderDocArgs};
use citum_io::RefsFormat as EngineRefsFormat;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::error::Error;

/// Standard CLI result type. Most subcommands return `()`; helpers that return
/// a value parameterise `T`.
pub(super) type CliResult<T = ()> = Result<T, Box<dyn Error>>;

impl From<RefsFormat> for EngineRefsFormat {
    fn from(format: RefsFormat) -> Self {
        match format {
            RefsFormat::CitumYaml => EngineRefsFormat::CitumYaml,
            RefsFormat::CitumJson => EngineRefsFormat::CitumJson,
            RefsFormat::CitumCbor => EngineRefsFormat::CitumCbor,
            RefsFormat::CslJson => EngineRefsFormat::CslJson,
            RefsFormat::Biblatex => EngineRefsFormat::Biblatex,
            RefsFormat::Ris => EngineRefsFormat::Ris,
        }
    }
}

pub(crate) fn run() -> CliResult {
    match Cli::parse().command {
        Commands::Render { command } => render::dispatch(command),
        Commands::Check(args) => check::run_check(args),
        Commands::Convert { command } => convert::dispatch(command),
        Commands::Registry { command } => registry::dispatch(command),
        Commands::Style { command } => style::dispatch(command),
        Commands::Locale { command } => locale::dispatch(command),
        Commands::Doctor { json } => doctor::run_doctor(json),
        #[cfg(feature = "schema")]
        Commands::Schema(args) => schema::run_schema(args),
        #[cfg(feature = "typescript")]
        Commands::Bindings(args) => bindings::run_bindings(args),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
        Commands::Doc(args) => run_deprecated_doc(args),
        Commands::Validate(args) => run_deprecated_validate(args),
    }
}

fn run_deprecated_doc(args: crate::args::LegacyDocArgs) -> CliResult {
    eprintln!("Warning: `citum doc` is deprecated. Use `citum render doc` with positional input.");
    render::run_render_doc(RenderDocArgs {
        input: args.document,
        style: args.style.display().to_string(),
        bibliography: vec![args.references],
        citations: Vec::new(),
        input_format: InputFormat::Djot,
        format: args.format,
        output: None,
        pdf: false,
        typst_keep_source: false,
        no_semantics: false,
    })
}

fn run_deprecated_validate(args: crate::args::LegacyValidateArgs) -> CliResult {
    eprintln!("Warning: `citum validate` is deprecated. Use `citum check --style`.");
    check::run_check(CheckArgs {
        style: Some(args.path.display().to_string()),
        bibliography: Vec::new(),
        citations: Vec::new(),
        json: false,
        strict: false,
    })
}
