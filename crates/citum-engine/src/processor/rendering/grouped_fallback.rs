//! Shared parameter types for grouped citation rendering.

/// Parameters for grouped citation rendering.
///
/// Bundles together related parameters that would otherwise lead to
/// `too_many_arguments` lint violations. This struct is used to pass
/// consistent rendering configuration to fallback functions.
#[derive(Debug)]
pub(crate) struct GroupRenderParams<'a> {
    /// The citation spec containing templates and configuration.
    pub(crate) spec: &'a citum_schema::CitationSpec,
    /// The citation mode (integral or non-integral).
    pub(crate) mode: &'a citum_schema::citation::CitationMode,
    /// Delimiter between items within a citation.
    pub(crate) intra_delimiter: &'a str,
    /// Whether to suppress author output.
    pub(crate) suppress_author: bool,
    /// The citation position (e.g., ibid, subsequent).
    pub(crate) position: Option<&'a citum_schema::citation::Position>,
}

/// Parameters for rendering a template with a citation number.
///
/// Bundles all rendering configuration into a single struct so that
/// `process_template_with_number` and its format-generic variant can
/// accept a single argument instead of ten, eliminating the need for
/// `#[allow(clippy::too_many_arguments)]` suppressions.
#[derive(Debug)]
pub struct TemplateRenderParams<'a> {
    /// The template components to render.
    pub template: &'a [citum_schema::template::TemplateComponent],
    /// The rendering context (citation or bibliography).
    pub context: crate::values::RenderContext,
    /// The citation mode (integral or non-integral).
    pub mode: citum_schema::citation::CitationMode,
    /// Whether to suppress the author component.
    pub suppress_author: bool,
    /// The citation number for numeric styles.
    pub citation_number: usize,
    /// The raw citation locator if present.
    pub locator_raw: Option<&'a citum_schema::citation::CitationLocator>,
    /// The citation position (e.g., ibid, subsequent).
    pub position: Option<&'a citum_schema::citation::Position>,
    /// Whether the author was rendered in integral form in the prose anchor.
    pub integral_name_state: Option<citum_schema::citation::IntegralNameState>,
}
