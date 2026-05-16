/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Registry I/O, configuration, and `registry` subcommands.

use super::CliResult;
use super::catalog::style_entry_kind;
use super::util::{confirm, validate_resource_name};
use crate::args::RegistryCommands;
use crate::table::build_table;
use citum_store::{StoreConfig, platform_config_dir};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct RegistrySourceRecord {
    pub(super) name: String,
    pub(super) source: String,
}

#[derive(Clone, Debug)]
pub(super) struct LoadedRegistry {
    pub(super) name: String,
    pub(super) source: String,
    pub(super) registry: citum_schema::StyleRegistry,
}

#[derive(Serialize)]
pub(super) struct RegistryInfo {
    pub(super) name: String,
    pub(super) source: String,
    pub(super) version: String,
    pub(super) styles: usize,
    pub(super) status: String,
}

pub(super) fn configured_registry_dir() -> Result<PathBuf, Box<dyn Error>> {
    platform_config_dir()
        .map(|dir| dir.join("registries"))
        .ok_or_else(|| "Unable to determine Citum config directory".into())
}

fn registry_sources_path() -> Result<PathBuf, Box<dyn Error>> {
    platform_config_dir()
        .map(|dir| dir.join("registry-sources.json"))
        .ok_or_else(|| "Unable to determine Citum config directory".into())
}

pub(super) fn read_registry_source_records() -> Result<Vec<RegistrySourceRecord>, Box<dyn Error>> {
    let path = registry_sources_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_registry_source_records(records: &[RegistrySourceRecord]) -> CliResult {
    let path = registry_sources_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(records)?)?;
    Ok(())
}

pub(super) fn registry_file_path(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    Ok(configured_registry_dir()?.join(format!("{name}.yaml")))
}

fn parse_http_url(source: &str) -> Result<Option<url::Url>, Box<dyn Error>> {
    if !source.starts_with("http://") && !source.starts_with("https://") {
        return Ok(None);
    }

    match url::Url::parse(source) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => Ok(Some(url)),
        Ok(url) => Err(format!("unsupported registry URL scheme '{}'", url.scheme()).into()),
        Err(err) => Err(format!("invalid registry URL: {err}").into()),
    }
}

fn fetch_registry_bytes(source: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if parse_http_url(source)?.is_some() {
        let resolver = citum_store::HttpResolver::from_platform_cache_dir()
            .ok_or("Unable to determine platform cache directory")?;
        return resolver.fetch_bytes(source);
    }
    Ok(fs::read(source)?)
}

fn parse_registry_bytes(bytes: &[u8]) -> Result<citum_schema::StyleRegistry, Box<dyn Error>> {
    let registry: citum_schema::StyleRegistry = serde_yaml::from_slice(bytes)?;
    registry.validate_sources()?;
    Ok(registry)
}

fn infer_registry_name(source: &str) -> Result<String, Box<dyn Error>> {
    if let Some(url) = parse_http_url(source)? {
        return url
            .host_str()
            .map(|host| host.replace('.', "-"))
            .ok_or_else(|| format!("URL has no host: {source}").into());
    }
    let path = Path::new(source);
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToString::to_string)
        .ok_or_else(|| format!("cannot infer registry name from {source}").into())
}

fn load_configured_registries() -> Result<Vec<LoadedRegistry>, Box<dyn Error>> {
    let records = read_registry_source_records()?;
    let mut registries = Vec::new();
    for record in records {
        let path = registry_file_path(&record.name)?;
        if !path.exists() {
            continue;
        }
        let registry = citum_schema::StyleRegistry::load_from_file(&path)?;
        registries.push(LoadedRegistry {
            name: record.name,
            source: record.source,
            registry,
        });
    }
    Ok(registries)
}

fn load_local_registry() -> Option<LoadedRegistry> {
    let path = Path::new("citum-registry.yaml");
    if !path.exists() {
        return None;
    }
    let registry = citum_schema::StyleRegistry::load_from_file(path).ok()?;
    Some(LoadedRegistry {
        name: "local".to_string(),
        source: path.display().to_string(),
        registry,
    })
}

pub(super) fn load_registry_chain() -> Result<Vec<LoadedRegistry>, Box<dyn Error>> {
    let mut registries = Vec::new();
    if let Some(local) = load_local_registry() {
        registries.push(local);
    }
    registries.extend(load_configured_registries()?);
    registries.push(LoadedRegistry {
        name: "embedded".to_string(),
        source: "embedded".to_string(),
        registry: citum_schema::embedded::default_registry(),
    });
    Ok(registries)
}

pub(super) fn dispatch(command: RegistryCommands) -> CliResult {
    match command {
        RegistryCommands::List { format } => run_registry_list(&format),
        RegistryCommands::Add { source, name } => run_registry_add(&source, name.as_deref()),
        RegistryCommands::Remove { name, yes } => run_registry_remove(&name, yes),
        RegistryCommands::Update { name, all } => run_registry_update(name.as_deref(), all),
        RegistryCommands::Resolve { name } => run_registry_resolve(&name),
    }
}

fn run_registry_list(format: &str) -> CliResult {
    let mut registries = Vec::new();
    let default_reg = citum_schema::embedded::default_registry();
    registries.push(RegistryInfo {
        name: "embedded".to_string(),
        source: "embedded".to_string(),
        version: default_reg.version.clone(),
        styles: default_reg.styles.len(),
        status: "ok".to_string(),
    });
    if let Some(local) = load_local_registry() {
        registries.push(RegistryInfo {
            name: local.name,
            source: local.source,
            version: local.registry.version,
            styles: local.registry.styles.len(),
            status: "ok".to_string(),
        });
    }
    for record in read_registry_source_records()? {
        let path = registry_file_path(&record.name)?;
        match citum_schema::StyleRegistry::load_from_file(&path) {
            Ok(registry) => registries.push(RegistryInfo {
                name: record.name,
                source: record.source,
                version: registry.version,
                styles: registry.styles.len(),
                status: "ok".to_string(),
            }),
            Err(err) => registries.push(RegistryInfo {
                name: record.name,
                source: record.source,
                version: "-".to_string(),
                styles: 0,
                status: err.to_string(),
            }),
        }
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&registries)?);
    } else {
        let rows = registries
            .iter()
            .map(|reg| {
                vec![
                    reg.name.clone(),
                    reg.source.clone(),
                    reg.version.clone(),
                    reg.styles.to_string(),
                    reg.status.clone(),
                ]
            })
            .collect();
        println!(
            "{}",
            build_table(&["Name", "Source", "Version", "Styles", "Status"], rows)
        );
    }
    Ok(())
}

fn run_registry_add(source: &str, name: Option<&str>) -> CliResult {
    let name = name.map_or_else(|| infer_registry_name(source), |name| Ok(name.to_string()))?;
    validate_resource_name(&name)?;
    let bytes = fetch_registry_bytes(source)?;
    let registry = parse_registry_bytes(&bytes)?;
    fs::create_dir_all(configured_registry_dir()?)?;
    fs::write(registry_file_path(&name)?, bytes)?;

    let mut records = read_registry_source_records()?;
    records.retain(|record| record.name != name);
    records.push(RegistrySourceRecord {
        name: name.clone(),
        source: source.to_string(),
    });
    write_registry_source_records(&records)?;

    println!(
        "Added registry '{name}' with {} styles.",
        registry.styles.len()
    );
    Ok(())
}

fn run_registry_remove(name: &str, yes: bool) -> CliResult {
    validate_resource_name(name)?;
    let path = registry_file_path(name)?;
    if !path.exists() {
        return Err(format!("configured registry not found: {name}").into());
    }
    if !yes && !confirm(&format!("Remove registry '{name}'?"))? {
        return Ok(());
    }
    fs::remove_file(path)?;
    let mut records = read_registry_source_records()?;
    records.retain(|record| record.name != name);
    write_registry_source_records(&records)?;
    println!("Removed registry: {name}");
    Ok(())
}

fn run_registry_update(name: Option<&str>, all: bool) -> CliResult {
    if name.is_some() == all {
        return Err("Specify either a registry name or --all.".into());
    }
    let mut records = read_registry_source_records()?;
    let selected: Vec<_> = records
        .iter()
        .filter(|record| all || Some(record.name.as_str()) == name)
        .cloned()
        .collect();

    // When all=true, also bootstrap config.yaml entries not yet in registry-sources.json
    if all && let Ok(config) = StoreConfig::load() {
        let existing_names: std::collections::HashSet<_> =
            records.iter().map(|r| r.name.clone()).collect();
        for registry_config in &config.registries {
            if !existing_names.contains(&registry_config.name)
                && let Ok(bytes) = fetch_registry_bytes(&registry_config.url)
                && let Ok(registry) = parse_registry_bytes(&bytes)
            {
                fs::create_dir_all(configured_registry_dir()?)?;
                fs::write(registry_file_path(&registry_config.name)?, &bytes)?;
                println!(
                    "Bootstrapped registry '{}' ({} styles).",
                    registry_config.name,
                    registry.styles.len()
                );
                records.push(RegistrySourceRecord {
                    name: registry_config.name.clone(),
                    source: registry_config.url.clone(),
                });
            }
        }
    }

    if selected.is_empty() && records.is_empty() {
        return Err("No configured registries matched.".into());
    }
    for record in selected {
        let bytes = fetch_registry_bytes(&record.source)?;
        let registry = parse_registry_bytes(&bytes)?;
        fs::write(registry_file_path(&record.name)?, bytes)?;
        println!(
            "Updated registry '{}' ({} styles).",
            record.name,
            registry.styles.len()
        );
    }

    // Write updated records to persist any newly bootstrapped entries
    if all {
        write_registry_source_records(&records)?;
    }

    Ok(())
}

fn run_registry_resolve(name: &str) -> CliResult {
    for loaded in load_registry_chain()? {
        if let Some(entry) = loaded.registry.resolve(name) {
            println!(
                "{} (registry:{}, {})",
                entry.id,
                loaded.name,
                style_entry_kind(entry)
            );
            return Ok(());
        }
    }
    Err(format!("style not found: {name}").into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn parse_http_url_treats_windows_absolute_paths_as_filesystem_paths() {
        let parsed = parse_http_url(r"C:\Users\citum\registry.yaml")
            .expect("windows path should not be treated as a URL");

        assert!(parsed.is_none());
    }

    #[test]
    fn test_registry_source_record_serialization() {
        let record = RegistrySourceRecord {
            name: "test".to_string(),
            source: "https://example.com/registry.yaml".to_string(),
        };
        let json = serde_json::to_string(&record).expect("should serialize");
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"source\":\"https://example.com/registry.yaml\""));
    }
}
