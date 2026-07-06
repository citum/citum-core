/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Bibliography grouping support.
//!
//! This module provides functionality for dividing bibliographies into
//! labeled groups via selector evaluation.

pub mod selector;

pub use selector::SelectorEvaluator;
