/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! `bindings` subcommand: export language bindings (TypeScript only, today).

use super::CliResult;
use crate::args::BindingsArgs;
use std::fs;

pub(super) fn run_bindings(args: BindingsArgs) -> CliResult {
    fs::create_dir_all(&args.out_dir)?;
    let out_path = args.out_dir.join("citum.d.ts");
    citum_bindings::export_typescript(&out_path).map_err(|e| format!("{e}"))?;
    println!("TypeScript bindings exported to {}", out_path.display());
    Ok(())
}
