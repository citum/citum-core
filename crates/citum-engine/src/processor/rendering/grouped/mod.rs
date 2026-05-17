/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Grouped citation rendering.

pub(super) mod component_predicates;
pub(super) mod core;
pub(super) mod grouping;
pub(super) mod sentence_initial;
pub(super) mod template_policy;

pub(super) use grouping::group_citation_items_by_author;
