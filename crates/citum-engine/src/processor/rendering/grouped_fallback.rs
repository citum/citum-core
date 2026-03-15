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
