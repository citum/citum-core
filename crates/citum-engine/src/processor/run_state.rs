/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Per-render-run mutable state for [`Processor`](super::Processor).
//!
//! This module is the first step of the migration described in
//! `docs/specs/EXPLICIT_RENDER_RUN_STATE.md`: it relocates the seven
//! `RefCell` fields that were previously declared directly on `Processor`
//! into their own type, `RunState`, with all existing call sites updated to
//! go through `self.run_state` (or `processor.run_state` in tests). No
//! method signatures change in this step — `Processor` still owns exactly
//! one `RunState` for its lifetime, so behavior is identical to before.
//!
//! Subsequent migration steps (see the spec) will thread `&mut RunState`
//! explicitly through registration methods and introduce a `FinalizedRun`
//! typestate so the ordering contract between citation registration and
//! bibliography/citation-number-dependent rendering is enforced by the type
//! system rather than by doc comments.

use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// Mutable per-render-run state: citation numbering, cite-order tracking,
/// and dynamic (cite-time) compound-group membership.
///
/// See `docs/specs/EXPLICIT_RENDER_RUN_STATE.md` for the full design and
/// migration plan.
#[derive(Debug)]
pub struct RunState {
    /// Citation numbers assigned to references (for numeric styles).
    pub(super) citation_numbers: RefCell<HashMap<String, usize>>,
    /// IDs of items that were cited in a visible way.
    pub(super) cited_ids: RefCell<HashSet<String>>,
    /// Compound numeric groups: citation number → ordered ref IDs in the group.
    pub(super) compound_groups: RefCell<IndexMap<usize, Vec<String>>>,
    /// Dynamic equivalent of `Processor::compound_set_by_ref` for cite-time groups.
    ///
    /// Maps each dynamic group member (head and tails) to the head's ref ID,
    /// which acts as the set identifier. Merged with static data at render time.
    pub(super) dynamic_compound_set_by_ref: RefCell<HashMap<String, String>>,
    /// Dynamic equivalent of `Processor::compound_member_index` for cite-time groups.
    ///
    /// Maps each dynamic group member to its 0-based position within the group.
    /// Merged with static data at render time.
    pub(super) dynamic_compound_member_index: RefCell<HashMap<String, usize>>,
    /// Dynamic equivalent of `Processor::compound_sets` for cite-time groups.
    ///
    /// Maps each dynamic group's head ref ID to the ordered list of all members.
    /// Merged with static `compound_sets` at render time so sub-label lookup works.
    pub(super) dynamic_compound_sets: RefCell<IndexMap<String, Vec<String>>>,
    /// First note number in which each reference was cited (note styles only).
    /// Populated during `normalize_note_context`; keyed by reference ID.
    pub(super) first_note_by_id: RefCell<HashMap<String, u32>>,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            citation_numbers: RefCell::new(HashMap::new()),
            cited_ids: RefCell::new(HashSet::new()),
            compound_groups: RefCell::new(IndexMap::new()),
            dynamic_compound_set_by_ref: RefCell::new(HashMap::new()),
            dynamic_compound_member_index: RefCell::new(HashMap::new()),
            dynamic_compound_sets: RefCell::new(IndexMap::new()),
            first_note_by_id: RefCell::new(HashMap::new()),
        }
    }
}
