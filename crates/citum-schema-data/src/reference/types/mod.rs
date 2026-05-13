/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Reference data types and structures for citation items.
//!
//! Split into submodules by domain:
//!
//! - [`common`]: shared primitives (strings, titles, dates, archive metadata)
//! - [`structural`]: hierarchical work types (monographs, serials, collections)
//! - [`legal`]: legal document types (cases, statutes, treaties, etc.)
//! - [`specialized`]: specialized work types (classic, patent, dataset, standard, software)

pub mod common;
pub mod legal;
pub mod specialized;
pub mod structural;

pub use common::*;
pub use legal::*;
pub use specialized::*;
pub use structural::*;
