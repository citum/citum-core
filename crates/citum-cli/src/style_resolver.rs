use citum_engine::{Processor, io::LoadedBibliography};
use citum_schema::{Locale, Style, locale::types::LocaleOverride};
use citum_store::{StoreConfig, StoreResolver, platform_data_dir};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn load_locale_file(path: &Path) -> Result<Locale, Box<dyn Error>> {
    Locale::from_file(path)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err).into())
}

/// Construct a [`Processor`] from a style, bibliography, and optional locale.
///
/// When `locale_override` is supplied it takes precedence over the style's
/// `default_locale`. Otherwise the locale is resolved first from disk (for
/// file-based styles) and then from embedded data, falling back to the
/// hardcoded `en-US` defaults.
pub(crate) fn create_processor(
    style: Style,
    loaded: LoadedBibliography,
    style_input: &str,
    no_semantics: bool,
    locale_override: Option<&str>,
) -> Result<Processor, Box<dyn Error>> {
    let LoadedBibliography { references, sets } = loaded;
    let compound_sets = sets.unwrap_or_default();
    let effective_locale = locale_override
        .map(str::to_owned)
        .or_else(|| style.info.default_locale.clone());
    if let Some(ref locale_id) = effective_locale {
        let path = Path::new(style_input);
        let mut locale = if path.exists() && path.is_file() {
            // File-based style: search for locale on disk, fall back to embedded.
            let locales_dir = find_locales_dir(style_input);
            let disk_locale = Locale::load(locale_id, &locales_dir);
            if disk_locale.locale == *locale_id || locale_id == "en-US" {
                disk_locale
            } else {
                load_locale_builtin(locale_id)
            }
        } else {
            // Builtin style: use embedded locale directly.
            load_locale_builtin(locale_id)
        };
        if locale_override.is_some() && locale.locale != *locale_id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("locale not found: '{locale_id}'"),
            )
            .into());
        }
        if let Some(override_id) = locale_override
            .is_none()
            .then(|| {
                style
                    .options
                    .as_ref()
                    .and_then(|options| options.locale_override.as_deref())
            })
            .flatten()
        {
            let locale_override = if path.exists() && path.is_file() {
                load_locale_override_for_file_style(override_id, style_input)?
                    .or_else(|| load_locale_override_builtin(override_id))
            } else {
                load_locale_override_builtin(override_id)
            };
            let locale_override = locale_override.ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                    "locale override not found: '{override_id}' (expected under locales/overrides/)"
                    ),
                )
            })?;
            locale.apply_override(&locale_override);
        }
        let mut processor =
            Processor::try_with_locale_and_compound_sets(style, references, locale, compound_sets)?;
        processor.show_semantics = !no_semantics;
        Ok(processor)
    } else {
        let mut processor = Processor::try_with_compound_sets(style, references, compound_sets)?;
        processor.show_semantics = !no_semantics;
        Ok(processor)
    }
}

/// Load a style from a file path, user store, or fallback to builtin name/alias.
pub(crate) fn load_any_style(
    style_input: &str,
    _no_semantics: bool,
) -> Result<Style, Box<dyn Error>> {
    use citum_store::resolver::{
        ChainResolver, EmbeddedResolver, FileResolver, HttpResolver, RegistryResolver,
        StyleResolver,
    };

    let mut resolvers: Vec<Box<dyn StyleResolver>> = vec![Box::new(FileResolver)];

    // Try user store if it exists
    if let Some(data_dir) = platform_data_dir()
        && data_dir.exists()
    {
        let config = StoreConfig::load().unwrap_or_default();
        resolvers.push(Box::new(StoreResolver::new(
            data_dir,
            config.store_format(),
        )));
    }

    if let Some(http) = HttpResolver::from_platform_cache_dir() {
        resolvers.push(Box::new(http));
    }

    let registry_resolver = RegistryResolver::new(citum_schema::embedded::default_registry())
        .with_base_dir(std::env::current_dir()?);
    let registry_resolver = if let Some(http) = HttpResolver::from_platform_cache_dir() {
        registry_resolver.with_http(http)
    } else {
        registry_resolver
    };
    resolvers.push(Box::new(registry_resolver));

    resolvers.push(Box::new(EmbeddedResolver));

    let chain = ChainResolver::new(resolvers);

    match chain.resolve_style(style_input) {
        Ok(style) => Ok(style),
        Err(citum_store::resolver::ResolverError::StyleNotFound(_)) => {
            let registry = citum_schema::embedded::default_registry();
            let candidates: Vec<_> = registry
                .styles
                .iter()
                .flat_map(|entry| {
                    std::iter::once(entry.id.as_str())
                        .chain(entry.aliases.iter().map(String::as_str))
                })
                .collect();
            let suggestions: Vec<_> = candidates
                .iter()
                .filter(|&&name| strsim::jaro_winkler(style_input, name) > 0.8)
                .collect();

            let mut msg = format!("style not found: '{style_input}'");
            if suggestions.is_empty() {
                msg.push_str("\n\nUse `citum style list` to see all available styles.");
            } else {
                msg.push_str("\n\nDid you mean one of these?");
                for s in suggestions {
                    msg.push_str("\n  - ");
                    msg.push_str(s);
                }
            }
            Err(msg.into())
        }
        Err(err) => Err(err.into()),
    }
}

/// Heuristically locate the `locales/` directory relative to a style file.
///
/// Checks the style's own directory and up to two parent directories, then falls
/// back to a `locales/` folder in the current working directory.  Returns `"."`
/// if no matching directory is found.
pub(crate) fn find_locales_dir(style_path: &str) -> PathBuf {
    let style_dir = Path::new(style_path).parent().unwrap_or(Path::new("."));
    let candidates = [
        style_dir.join("locales"),
        style_dir.join("../locales"),
        style_dir.join("../../locales"),
        PathBuf::from("locales"),
    ];

    for candidate in &candidates {
        if candidate.exists() && candidate.is_dir() {
            return candidate.clone();
        }
    }

    PathBuf::from(".")
}

pub(crate) fn load_locale_override_for_file_style(
    override_id: &str,
    style_path: &str,
) -> Result<Option<LocaleOverride>, Box<dyn Error>> {
    let overrides_dir = find_locales_dir(style_path).join("overrides");
    load_locale_override_from_dir(override_id, &overrides_dir)
}

/// Load a locale from embedded bytes, falling back to en-US.
pub(crate) fn load_locale_builtin(locale_id: &str) -> Locale {
    if let Some(bytes) = citum_schema::embedded::get_locale_bytes(locale_id) {
        let content = String::from_utf8_lossy(bytes);
        Locale::from_yaml_str(&content).unwrap_or_else(|_| Locale::en_us())
    } else {
        // Locale not bundled — fall back to the hardcoded en-US default.
        Locale::en_us()
    }
}

pub(crate) fn load_locale_override_from_dir(
    override_id: &str,
    overrides_dir: &Path,
) -> Result<Option<LocaleOverride>, Box<dyn Error>> {
    for ext in ["yaml", "yml", "json", "cbor"] {
        let path = overrides_dir.join(format!("{override_id}.{ext}"));
        if path.exists() && path.is_file() {
            return load_locale_override_file(&path).map(Some);
        }
    }
    Ok(None)
}

pub(crate) fn load_locale_override_file(path: &Path) -> Result<LocaleOverride, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");
    parse_locale_override_bytes(&bytes, ext)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err).into())
}

pub(crate) fn load_locale_override_builtin(override_id: &str) -> Option<LocaleOverride> {
    let bytes = citum_schema::embedded::get_locale_override_bytes(override_id)?;
    parse_locale_override_bytes(bytes, "yaml").ok()
}

pub(crate) fn parse_locale_override_bytes(
    bytes: &[u8],
    ext: &str,
) -> Result<LocaleOverride, String> {
    use citum_schema::locale::raw::RawLocaleOverride;

    match ext {
        "cbor" => ciborium::de::from_reader::<RawLocaleOverride, _>(std::io::Cursor::new(bytes))
            .map(Into::into)
            .map_err(|e| format!("Failed to parse CBOR locale override: {e}")),
        "json" => serde_json::from_slice::<RawLocaleOverride>(bytes)
            .map(Into::into)
            .map_err(|e| format!("Failed to parse JSON locale override: {e}")),
        _ => serde_yaml::from_slice::<RawLocaleOverride>(bytes)
            .map(Into::into)
            .map_err(|e| format!("Failed to parse YAML locale override: {e}")),
    }
}
