/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Embedded baseline locale data.
//!
//! Houses the hardcoded en-US fallback that every locale inherits from. Other
//! language baselines are loaded from YAML at runtime via
//! [`crate::locale::Locale::load`].

pub(crate) mod en_us;

pub(crate) use en_us::{
    embedded_en_us_vocab, en_us_archive_messages, en_us_locator_terms, en_us_role_terms,
};
