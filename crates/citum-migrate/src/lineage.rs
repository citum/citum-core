/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Migration-time style lineage and wrapper classification.

use citum_schema::Style;
use citum_schema::embedded;
use citum_schema::registry::{StyleKind, StyleRegistry};
use serde_yaml::{Mapping, Value};
use std::fmt;
use std::fs;
use std::path::Path;

/// Taxonomy-level semantic class for the migration target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticClass {
    Base,
    Profile,
    Journal,
    Independent,
    Unknown,
}

/// Current implementation form derived from the checked-in style shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationForm {
    Alias,
    ConfigWrapper,
    StructuralWrapper,
    Standalone,
    Unknown,
}

/// Migration-time lineage for one style target.
#[derive(Debug, Clone)]
pub struct StyleLineage {
    /// Canonical style ID derived from the CSL filename.
    pub style_id: String,
    /// Semantic class from the registry or alias surface.
    pub semantic_class: SemanticClass,
    /// Current implementation form derived from the checked-in style.
    pub implementation_form: ImplementationForm,
    /// Established parent style ID, if any.
    pub parent_style_id: Option<String>,
    parent_style: Option<Style>,
}

/// Failure while resolving or rewriting migration lineage.
#[derive(Debug)]
pub enum LineageError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
}

impl fmt::Display for LineageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineageError::Io(err) => write!(f, "{err}"),
            LineageError::Yaml(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for LineageError {}

impl From<std::io::Error> for LineageError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_yaml::Error> for LineageError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Yaml(value)
    }
}

impl StyleLineage {
    /// Resolve migration lineage from the CSL file path and repository root.
    ///
    /// # Errors
    ///
    /// Returns an error when the current style or its established parent cannot
    /// be read or parsed from repo-owned YAML.
    pub fn resolve(input_path: &str, repo_root: &Path) -> Result<Self, LineageError> {
        let style_id = Path::new(input_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown")
            .to_string();

        let registry = StyleRegistry::load_default();
        let exact_entry = registry.styles.iter().find(|entry| entry.id == style_id);
        let alias_target = registry
            .styles
            .iter()
            .find(|entry| entry.aliases.iter().any(|alias| alias == &style_id));

        let current_style = load_current_style(repo_root, &style_id, exact_entry)?;
        let semantic_class = if let Some(entry) = exact_entry {
            map_style_kind(entry.kind.as_ref())
        } else if alias_target.is_some() {
            SemanticClass::Journal
        } else if let Some(style) = current_style.as_ref() {
            if style.extends.is_some() {
                SemanticClass::Journal
            } else {
                SemanticClass::Independent
            }
        } else {
            SemanticClass::Unknown
        };
        let parent_style_id = current_style
            .as_ref()
            .and_then(|style| style.extends.as_ref())
            .map(|base| base.key().to_string())
            .or_else(|| alias_target.map(|entry| entry.id.clone()));
        let parent_style = parent_style_id
            .as_deref()
            .map(|parent| load_style_by_id(repo_root, parent))
            .transpose()?;

        let implementation_form = if current_style.is_none() && alias_target.is_some() {
            ImplementationForm::Alias
        } else if let Some(style) = current_style.as_ref() {
            derive_implementation_form(style)
        } else {
            ImplementationForm::Unknown
        };

        Ok(Self {
            style_id,
            semantic_class,
            implementation_form,
            parent_style_id,
            parent_style,
        })
    }

    /// Rewrite a standalone migrated style into wrapper form when the current
    /// repo truth already establishes that relationship.
    ///
    /// # Errors
    ///
    /// Returns an error when the rewritten wrapper cannot be serialized or
    /// deserialized as a valid `Style`.
    pub fn apply_to_migrated_style(&self, style: Style) -> Result<Style, LineageError> {
        let Some(parent_style_id) = self.parent_style_id.as_deref() else {
            return Ok(style);
        };
        let Some(parent_style) = self.parent_style.as_ref() else {
            return Ok(style);
        };

        let exclude_template_paths = match (self.semantic_class, self.implementation_form) {
            (
                SemanticClass::Profile | SemanticClass::Journal,
                ImplementationForm::ConfigWrapper,
            ) => true,
            (SemanticClass::Journal, ImplementationForm::StructuralWrapper) => false,
            _ => return Ok(style),
        };

        let child = serde_yaml::to_value(&style)?;
        let parent = serde_yaml::to_value(parent_style.clone().into_resolved())?;

        let mut diff = match diff_value(&child, &parent, &mut Vec::new(), exclude_template_paths) {
            Some(Value::Mapping(map)) => map,
            Some(other) => {
                let mut map = Mapping::new();
                map.insert(Value::String("style".to_string()), other);
                map
            }
            None => Mapping::new(),
        };

        diff.insert(
            Value::String("extends".to_string()),
            Value::String(parent_style_id.to_string()),
        );

        let mut rebuilt: Style = serde_yaml::from_value(Value::Mapping(diff))?;
        rebuilt.raw_yaml = None;
        Ok(rebuilt)
    }
}

fn map_style_kind(kind: Option<&StyleKind>) -> SemanticClass {
    match kind {
        Some(StyleKind::Base) => SemanticClass::Base,
        Some(StyleKind::Profile) => SemanticClass::Profile,
        Some(StyleKind::Journal) => SemanticClass::Journal,
        Some(StyleKind::Independent) => SemanticClass::Independent,
        None => SemanticClass::Unknown,
    }
}

fn load_current_style(
    repo_root: &Path,
    style_id: &str,
    exact_entry: Option<&citum_schema::RegistryEntry>,
) -> Result<Option<Style>, LineageError> {
    let local_path = repo_root.join("styles").join(format!("{style_id}.yaml"));
    if local_path.exists() {
        return Ok(Some(load_style_from_path(&local_path)?));
    }

    if let Some(entry) = exact_entry
        && let Some(name) = &entry.builtin
        && let Some(style) = embedded::get_embedded_style(name)
    {
        return Ok(Some(style?));
    }

    Ok(None)
}

fn load_style_by_id(repo_root: &Path, style_id: &str) -> Result<Style, LineageError> {
    let local_path = repo_root.join("styles").join(format!("{style_id}.yaml"));
    if local_path.exists() {
        return load_style_from_path(&local_path);
    }

    if let Some(style) = embedded::get_embedded_style(style_id) {
        return Ok(style?);
    }

    let embedded_path = repo_root
        .join("styles")
        .join("embedded")
        .join(format!("{style_id}.yaml"));
    if embedded_path.exists() {
        return load_style_from_path(&embedded_path);
    }

    Err(LineageError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("unable to resolve parent style `{style_id}`"),
    )))
}

fn load_style_from_path(path: &Path) -> Result<Style, LineageError> {
    let yaml = fs::read_to_string(path)?;
    Ok(Style::from_yaml_str(&yaml)?)
}

fn derive_implementation_form(style: &Style) -> ImplementationForm {
    if style.extends.is_none() {
        return ImplementationForm::Standalone;
    }

    if has_template_bearing_structure(style) {
        ImplementationForm::StructuralWrapper
    } else {
        ImplementationForm::ConfigWrapper
    }
}

fn has_template_bearing_structure(style: &Style) -> bool {
    if style.templates.is_some() || yaml_path_present(style.raw_yaml.as_ref(), &["templates"]) {
        return true;
    }

    TEMPLATE_BEARING_PATHS
        .iter()
        .any(|path| yaml_path_present(style.raw_yaml.as_ref(), path))
}

const TEMPLATE_BEARING_PATHS: [&[&str]; 9] = [
    &["templates"],
    &["citation", "template"],
    &["citation", "type-variants"],
    &["citation", "integral", "template"],
    &["citation", "integral", "type-variants"],
    &["citation", "non-integral", "template"],
    &["citation", "non-integral", "type-variants"],
    &["bibliography", "template"],
    &["bibliography", "type-variants"],
];

fn yaml_path_present(value: Option<&Value>, path: &[&str]) -> bool {
    let Some(mut current) = value else {
        return false;
    };
    for segment in path {
        let Value::Mapping(map) = current else {
            return false;
        };
        let key = Value::String((*segment).to_string());
        let Some(next) = map.get(&key) else {
            return false;
        };
        current = next;
    }
    true
}

fn diff_value(
    child: &Value,
    parent: &Value,
    path: &mut Vec<String>,
    exclude_template_paths: bool,
) -> Option<Value> {
    if child == parent {
        return None;
    }

    match (child, parent) {
        (Value::Mapping(child_map), Value::Mapping(parent_map)) => {
            let mut diff = Mapping::new();
            for (key, child_value) in child_map {
                let Some(segment) = key.as_str() else {
                    diff.insert(key.clone(), child_value.clone());
                    continue;
                };
                path.push(segment.to_string());
                if exclude_template_paths && is_template_bearing_path(path) {
                    path.pop();
                    continue;
                }

                match parent_map.get(key) {
                    Some(parent_value) => {
                        if let Some(child_diff) =
                            diff_value(child_value, parent_value, path, exclude_template_paths)
                        {
                            diff.insert(key.clone(), child_diff);
                        }
                    }
                    None => {
                        diff.insert(key.clone(), child_value.clone());
                    }
                }
                path.pop();
            }

            if diff.is_empty() {
                None
            } else {
                Some(Value::Mapping(diff))
            }
        }
        _ => Some(child.clone()),
    }
}

fn is_template_bearing_path(path: &[String]) -> bool {
    TEMPLATE_BEARING_PATHS.iter().any(|candidate| {
        candidate.len() == path.len()
            && candidate
                .iter()
                .zip(path.iter())
                .all(|(expected, actual)| *expected == actual)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::template::{
        ContributorForm, ContributorRole, DateForm, DateVariable, SimpleVariable,
        TemplateComponent, TemplateContributor, TemplateDate, TemplateVariable,
    };
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate dir should have workspace root")
            .to_path_buf()
    }

    fn minimal_migrated_style() -> Style {
        Style {
            info: citum_schema::StyleInfo {
                title: Some("Migrated Test".to_string()),
                id: Some("https://example.org/migrated-test".to_string()),
                ..Default::default()
            },
            citation: Some(citum_schema::CitationSpec {
                template: Some(vec![
                    TemplateComponent::Contributor(TemplateContributor {
                        contributor: ContributorRole::Author,
                        form: ContributorForm::Short,
                        ..Default::default()
                    }),
                    TemplateComponent::Date(TemplateDate {
                        date: DateVariable::Issued,
                        form: DateForm::Year,
                        ..Default::default()
                    }),
                ]),
                ..Default::default()
            }),
            bibliography: Some(citum_schema::BibliographySpec {
                template: Some(vec![
                    TemplateComponent::Contributor(TemplateContributor {
                        contributor: ContributorRole::Author,
                        form: ContributorForm::Long,
                        ..Default::default()
                    }),
                    TemplateComponent::Variable(TemplateVariable {
                        variable: SimpleVariable::Doi,
                        ..Default::default()
                    }),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn resolves_embedded_profile_as_config_wrapper() {
        let lineage =
            StyleLineage::resolve("styles-legacy/elsevier-harvard.csl", &repo_root()).unwrap();

        assert_eq!(lineage.style_id, "elsevier-harvard");
        assert_eq!(lineage.semantic_class, SemanticClass::Profile);
        assert_eq!(
            lineage.implementation_form,
            ImplementationForm::ConfigWrapper
        );
        assert_eq!(
            lineage.parent_style_id.as_deref(),
            Some("elsevier-harvard-core")
        );
    }

    #[test]
    fn resolves_journal_config_wrapper_from_local_style() {
        let lineage = StyleLineage::resolve(
            "styles-legacy/disability-and-rehabilitation.csl",
            &repo_root(),
        )
        .unwrap();

        assert_eq!(lineage.semantic_class, SemanticClass::Journal);
        assert_eq!(
            lineage.implementation_form,
            ImplementationForm::ConfigWrapper
        );
        assert_eq!(
            lineage.parent_style_id.as_deref(),
            Some("elsevier-with-titles")
        );
    }

    #[test]
    fn resolves_journal_structural_wrapper_from_local_style() {
        let lineage = StyleLineage::resolve(
            "styles-legacy/american-society-of-mechanical-engineers.csl",
            &repo_root(),
        )
        .unwrap();

        assert_eq!(lineage.semantic_class, SemanticClass::Journal);
        assert_eq!(
            lineage.implementation_form,
            ImplementationForm::StructuralWrapper
        );
        assert_eq!(lineage.parent_style_id.as_deref(), Some("ieee"));
    }

    #[test]
    fn resolves_unknown_style_as_unknown() {
        let lineage =
            StyleLineage::resolve("styles-legacy/definitely-unknown-style.csl", &repo_root())
                .unwrap();

        assert_eq!(lineage.semantic_class, SemanticClass::Unknown);
        assert_eq!(lineage.implementation_form, ImplementationForm::Unknown);
        assert!(lineage.parent_style_id.is_none());
    }

    #[test]
    fn config_wrapper_output_sets_extends_and_strips_templates() {
        let lineage =
            StyleLineage::resolve("styles-legacy/elsevier-harvard.csl", &repo_root()).unwrap();
        let rewritten = lineage
            .apply_to_migrated_style(minimal_migrated_style())
            .unwrap();

        assert_eq!(
            rewritten.extends.as_ref().map(|base| base.key()),
            Some("elsevier-harvard-core")
        );
        assert!(
            rewritten
                .citation
                .as_ref()
                .and_then(|citation| citation.template.as_ref())
                .is_none(),
            "config-wrapper profiles must not keep local citation templates"
        );
        assert!(
            rewritten
                .bibliography
                .as_ref()
                .and_then(|bibliography| bibliography.template.as_ref())
                .is_none(),
            "config-wrapper profiles must not keep local bibliography templates"
        );
    }

    #[test]
    fn structural_wrapper_output_keeps_extends_and_structural_deltas() {
        let lineage = StyleLineage::resolve(
            "styles-legacy/american-society-of-mechanical-engineers.csl",
            &repo_root(),
        )
        .unwrap();
        let rewritten = lineage
            .apply_to_migrated_style(minimal_migrated_style())
            .unwrap();

        assert_eq!(
            rewritten.extends.as_ref().map(|base| base.key()),
            Some("ieee")
        );
        assert!(
            rewritten
                .citation
                .as_ref()
                .and_then(|citation| citation.template.as_ref())
                .is_some(),
            "structural wrappers should preserve local structural citation deltas"
        );
    }
}
