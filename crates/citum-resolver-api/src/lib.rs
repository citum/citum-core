/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Canonical style resolution interfaces for the Citum citation engine.
//!
//! This crate provides a lightweight bridge between the schema layer
//! (which defines style models) and the store layer (which implements
//! persistence and network resolution). It minimizes dependencies
//! to facilitate integration by third-party tool authors who don't
//! need the full store logic.

/// Error types and conversion helpers.
pub mod error;
pub use error::{ResolutionError, ResolverError};

/// A resolver that can locate styles and locales.
///
/// This trait uses associated types to break cyclic dependencies between
/// schema models and resolution logic.
pub trait StyleResolver: Send + Sync {
    /// The style type resolved by this implementation.
    type Style;
    /// The locale type resolved by this implementation.
    type Locale;

    /// Resolve a style by URI or ID.
    ///
    /// # Errors
    /// Returns a [`ResolverError`] if the style cannot be found or loaded.
    fn resolve_style(&self, uri: &str) -> Result<Self::Style, ResolverError>;

    /// Resolve a locale by ID.
    ///
    /// # Errors
    /// Returns a [`ResolverError`] if the locale cannot be found or loaded.
    fn resolve_locale(&self, id: &str) -> Result<Self::Locale, ResolverError>;
}
