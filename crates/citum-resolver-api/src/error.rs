/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use std::borrow::Cow;
use thiserror::Error;

/// Error type for style resolution operations at the store layer.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ResolverError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid style: {0}")]
    InvalidStyle(Cow<'static, str>),
    #[error("style not found: {0}")]
    StyleNotFound(Cow<'static, str>),
    #[error("locale not found: {0}")]
    LocaleNotFound(Cow<'static, str>),
    #[error("yaml error: {0}")]
    YamlError(String),
    #[cfg(feature = "serde_json")]
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("cbor error: {0}")]
    CborError(String),
    #[cfg(feature = "http")]
    #[error("http error: {0}")]
    HttpError(String),
    #[cfg(feature = "http")]
    #[error("git error: {0}")]
    GitError(String),
    /// The URI host or origin is not in the resolver's allowlist.
    #[error("host not in resolver allowlist: {uri} ({reason})")]
    Denied { uri: String, reason: String },
    /// The style's `citum-version` is not compatible with the running engine.
    #[error(
        "engine version mismatch for {uri}: engine requires {required}, style declares {declared}"
    )]
    VersionMismatch {
        uri: String,
        required: String,
        declared: String,
    },
    /// The content does not match the expected integrity hash.
    #[error("integrity failure for {uri}: expected {expected}, got {actual}")]
    IntegrityFailure {
        uri: String,
        expected: String,
        actual: String,
    },
    /// Generic network or transport failure.
    #[error("network error fetching {uri}: {reason}")]
    NetworkError { uri: String, reason: String },
}

/// Error type for style resolution and inheritance processing at the schema layer.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ResolutionError {
    /// A `profile` style attempted to override template-bearing structure.
    #[error("profile styles may not override template-bearing field `{location}`")]
    InvalidProfileOverride {
        /// Human-readable location hint.
        location: String,
    },
    /// An inheritance loop was detected.
    #[error("inheritance loop detected at base `{base}`")]
    InheritanceLoop {
        /// Base key that closed the cycle.
        base: String,
    },
    /// A `file://` URI could not be resolved.
    #[error("failed to resolve URI `{uri}`: {reason}")]
    UriResolutionFailed {
        /// The URI that failed to resolve.
        uri: String,
        /// Reason for failure.
        reason: String,
    },
    /// A Template V3 variant references a missing parent variant.
    #[error("template variant `{location}` extends missing variant `{selector}`")]
    MissingTemplateVariantParent {
        /// Human-readable location hint.
        location: String,
        /// Parent selector that could not be found.
        selector: String,
    },
    /// A Template V3 variant parent chain contains a cycle.
    #[error("template variant inheritance loop at `{location}` through `{selector}`")]
    TemplateVariantCycle {
        /// Human-readable location hint.
        location: String,
        /// Selector that closed the cycle.
        selector: String,
    },
    /// A Template V3 operation matched no components.
    #[error("template variant operation in `{location}` matched no component")]
    TemplateVariantAnchorNotFound {
        /// Human-readable location hint.
        location: String,
    },
    /// A Template V3 operation matched more than one component.
    #[error("template variant operation in `{location}` matched multiple components")]
    TemplateVariantAmbiguousAnchor {
        /// Human-readable location hint.
        location: String,
    },
    /// A Template V3 add operation does not define exactly one anchor.
    #[error("template variant add operation in `{location}` must specify exactly one of before/after")]
    InvalidTemplateVariantAdd {
        /// Human-readable location hint.
        location: String,
    },
    /// The fetched parent style's content did not hash to the value declared
    /// in `extends-pin`.
    #[error("extends-pin integrity check failed for `{uri}`: expected {expected}, got {actual}")]
    IntegrityFailure {
        /// URI of the parent that failed integrity verification.
        uri: String,
        /// CID declared in the child's `extends-pin`.
        expected: String,
        /// CID computed from the bytes the resolver actually returned.
        actual: String,
    },
    /// The fetched style declares a `citum-version` requirement that the
    /// running engine does not satisfy.
    #[error("style `{uri}` requires citum-version `{required}`; running engine is `{declared}`")]
    VersionMismatch {
        /// URI whose `citum-version` requirement was unsatisfiable.
        uri: String,
        /// `citum-version` requirement declared by the style's `info` block.
        required: String,
        /// Version of the running engine.
        declared: String,
    },
}

impl ResolutionError {
    /// Convert a [`ResolverError`] into a [`ResolutionError`] for a specific URI.
    pub fn from_resolver_error(uri: &str, err: ResolverError) -> Self {
        match err {
            ResolverError::IntegrityFailure {
                expected, actual, ..
            } => ResolutionError::IntegrityFailure {
                uri: uri.into(),
                expected,
                actual,
            },
            ResolverError::VersionMismatch {
                required, declared, ..
            } => ResolutionError::VersionMismatch {
                uri: uri.into(),
                required,
                declared,
            },
            _ => ResolutionError::UriResolutionFailed {
                uri: uri.into(),
                reason: err.to_string(),
            },
        }
    }
}
