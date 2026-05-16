/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! `doctor` subcommand: environment and registry health summary.

use super::CliResult;
use super::registry::{RegistryInfo, read_registry_source_records, registry_file_path};
use citum_store::{
    StoreConfig, StoreResolver, platform_cache_dir, platform_config_dir, platform_data_dir,
};
use serde::Serialize;

#[derive(Serialize)]
struct DoctorReport {
    data_dir: Option<String>,
    config_dir: Option<String>,
    cache_dir: Option<String>,
    installed_styles: usize,
    installed_locales: usize,
    registries: Vec<RegistryInfo>,
}

pub(super) fn run_doctor(json: bool) -> CliResult {
    let data_dir = platform_data_dir();
    let config = StoreConfig::load().unwrap_or_default();
    let (installed_styles, installed_locales) = if let Some(dir) = data_dir.clone() {
        let resolver = StoreResolver::new(dir, config.store_format());
        (
            resolver.list_styles().unwrap_or_default().len(),
            resolver.list_locales().unwrap_or_default().len(),
        )
    } else {
        (0, 0)
    };
    let mut registries = Vec::new();
    let default_reg = citum_schema::embedded::default_registry();
    registries.push(RegistryInfo {
        name: "embedded".to_string(),
        source: "embedded".to_string(),
        version: default_reg.version,
        styles: default_reg.styles.len(),
        status: "ok".to_string(),
    });
    for record in read_registry_source_records().unwrap_or_default() {
        let path = registry_file_path(&record.name)?;
        let status = if path.exists() { "ok" } else { "missing" };
        registries.push(RegistryInfo {
            name: record.name,
            source: record.source,
            version: "-".to_string(),
            styles: 0,
            status: status.to_string(),
        });
    }
    let report = DoctorReport {
        data_dir: data_dir.map(|path| path.display().to_string()),
        config_dir: platform_config_dir().map(|path| path.display().to_string()),
        cache_dir: platform_cache_dir().map(|path| path.display().to_string()),
        installed_styles,
        installed_locales,
        registries,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Data dir:          {}",
            report.data_dir.as_deref().unwrap_or("-")
        );
        println!(
            "Config dir:        {}",
            report.config_dir.as_deref().unwrap_or("-")
        );
        println!(
            "Cache dir:         {}",
            report.cache_dir.as_deref().unwrap_or("-")
        );
        println!("Installed styles:  {}", report.installed_styles);
        println!("Installed locales: {}", report.installed_locales);
        println!("Registries:        {}", report.registries.len());
    }
    Ok(())
}
