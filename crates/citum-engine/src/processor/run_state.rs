/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Per-render-run mutable state for [`Processor`](super::Processor).
//!
//! See `docs/specs/EXPLICIT_RENDER_RUN_STATE.md` for the full design.
//!
//! `RunState` owns the citation-order-dependent state that used to live as
//! `RefCell` fields directly on `Processor`: citation numbers, the cited-ID
//! set, dynamic (cite-time) compound-group membership, and first-note
//! tracking. It is created fresh via [`Processor::begin_run`](super::Processor::begin_run),
//! populated in citation-processing order by registration methods that take
//! `&mut RunState`, and then finalized into a [`FinalizedRun`] so that
//! bibliography rendering — which takes `&FinalizedRun` — cannot be called
//! before registration is complete. That ordering contract is enforced by
//! the type system: there is no way to construct a `FinalizedRun` other than
//! through [`RunState::finalize`]. Citation rendering itself stays
//! `&mut RunState`-threaded rather than moving to `&FinalizedRun`:
//! registration and rendering are interleaved per citation (see
//! `citation.rs`'s module docs), so a citation can only be rendered as part
//! of the same in-progress run that is registering it.
//!
//! Two fields, `citation_numbers` and `first_note_by_id`, stay behind a
//! `RwLock` even inside `RunState`/`FinalizedRun`: the render layer
//! (`Renderer::get_or_assign_citation_number`) lazily assigns a citation
//! number the first time a reference is rendered, which is a monotonic,
//! assign-once operation, not a read. This does not weaken the ordering
//! contract this type adds — it only means "render before registration is
//! complete" is a compile error, not that rendering can never touch interior
//! state. `RwLock` (rather than `RefCell`) is required so that `FinalizedRun`
//! is `Sync` and bibliography entries can render across threads (see
//! `docs/specs/PARALLEL_BIBLIOGRAPHY_RENDERING.md`); lock poisoning is
//! recovered from rather than propagated, since a panicking reader/writer
//! does not invalidate the numbering data already in the map.

use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

/// Mutable per-render-run state: citation numbering, cite-order tracking,
/// and dynamic (cite-time) compound-group membership.
///
/// Create with [`Processor::begin_run`](super::Processor::begin_run);
/// populate via registration methods (`&self, &mut RunState`); consume via
/// [`finalize`](RunState::finalize) before rendering.
///
/// `Clone` is provided for long-lived callers (e.g. the FFI session handle)
/// that need to render a bibliography from a snapshot of the current state
/// without pausing ongoing citation registration on the original `RunState`.
/// Implemented by hand (rather than derived) because `RwLock<T>` is not
/// `Clone` even when `T` is; the impl below clones the locked contents into
/// fresh locks instead.
#[derive(Debug)]
pub struct RunState {
    /// Citation numbers assigned to references (for numeric styles).
    ///
    /// Stays `RwLock`: the render layer lazily assigns numbers the first
    /// time a reference is rendered (see module docs).
    pub(super) citation_numbers: RwLock<HashMap<String, usize>>,
    /// First note number in which each reference was cited (note styles only).
    ///
    /// Stays `RwLock` for the same reason as `citation_numbers`.
    pub(super) first_note_by_id: RwLock<HashMap<String, u32>>,
    /// IDs of items that were cited in a visible way.
    pub(super) cited_ids: HashSet<String>,
    /// Compound numeric groups: citation number → ordered ref IDs in the group.
    pub(super) compound_groups: IndexMap<usize, Vec<String>>,
    /// Dynamic equivalent of `Processor::compound_set_by_ref` for cite-time groups.
    ///
    /// Maps each dynamic group member (head and tails) to the head's ref ID,
    /// which acts as the set identifier. Merged with static data at render time.
    pub(super) dynamic_compound_set_by_ref: HashMap<String, String>,
    /// Dynamic equivalent of `Processor::compound_member_index` for cite-time groups.
    ///
    /// Maps each dynamic group member to its 0-based position within the group.
    /// Merged with static data at render time.
    pub(super) dynamic_compound_member_index: HashMap<String, usize>,
    /// Dynamic equivalent of `Processor::compound_sets` for cite-time groups.
    ///
    /// Maps each dynamic group's head ref ID to the ordered list of all members.
    /// Merged with static `compound_sets` at render time so sub-label lookup works.
    pub(super) dynamic_compound_sets: IndexMap<String, Vec<String>>,
}

impl Clone for RunState {
    /// Snapshot the current state, including the interior-mutable citation
    /// numbers and first-note map, into a fresh, independently-lockable copy.
    fn clone(&self) -> Self {
        Self {
            citation_numbers: RwLock::new(
                self.citation_numbers
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            ),
            first_note_by_id: RwLock::new(
                self.first_note_by_id
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            ),
            cited_ids: self.cited_ids.clone(),
            compound_groups: self.compound_groups.clone(),
            dynamic_compound_set_by_ref: self.dynamic_compound_set_by_ref.clone(),
            dynamic_compound_member_index: self.dynamic_compound_member_index.clone(),
            dynamic_compound_sets: self.dynamic_compound_sets.clone(),
        }
    }
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            citation_numbers: RwLock::new(HashMap::new()),
            first_note_by_id: RwLock::new(HashMap::new()),
            cited_ids: HashSet::new(),
            compound_groups: IndexMap::new(),
            dynamic_compound_set_by_ref: HashMap::new(),
            dynamic_compound_member_index: HashMap::new(),
            dynamic_compound_sets: IndexMap::new(),
        }
    }
}

impl RunState {
    /// Complete the registration phase, producing a [`FinalizedRun`].
    ///
    /// This is a plain newtype wrap with no additional computation; it
    /// exists purely as a compile-time marker that registration for this
    /// run is considered complete, so rendering methods that require
    /// citation order/numbering can require `&FinalizedRun` instead of
    /// `&RunState`.
    #[must_use]
    pub fn finalize(self) -> FinalizedRun {
        FinalizedRun(self)
    }
}

/// A [`RunState`] that has completed the registration phase.
///
/// Rendering methods that depend on cite order or citation numbers (e.g.
/// bibliography rendering, citation-collapse across a document) take
/// `&FinalizedRun` rather than `&RunState`, so calling them before
/// registration is complete is a compile error.
#[derive(Debug)]
pub struct FinalizedRun(pub(super) RunState);

impl FinalizedRun {
    /// Borrow the underlying run state.
    ///
    /// Available to processor submodules that need read access to run
    /// fields during rendering (e.g. `cited_ids`, `compound_groups`).
    pub(super) fn state(&self) -> &RunState {
        &self.0
    }
}
