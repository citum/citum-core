/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Grouped citation rendering.

pub(super) mod core;
pub(super) mod grouping;

pub(super) use grouping::group_citation_items_by_author;
