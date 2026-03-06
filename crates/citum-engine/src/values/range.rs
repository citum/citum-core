/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Shared helpers for collapsing ordered consecutive sequences into spans.

/// One collapsed segment from an ordered numeric sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsecutiveSegment {
    /// A single standalone value.
    Single(u32),
    /// A consecutive range from `start` to `end`, inclusive.
    Range {
        /// The first value in the consecutive range.
        start: u32,
        /// The last value in the consecutive range.
        end: u32,
    },
}

/// Collapse an ordered sequence into standalone values and consecutive ranges.
///
/// Duplicate values are coalesced, and descending steps start a new segment.
pub fn consecutive_segments(values: &[u32]) -> Vec<ConsecutiveSegment> {
    if values.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut start = values[0];
    let mut prev = values[0];

    for &value in &values[1..] {
        if value == prev {
            continue;
        }

        if value == prev + 1 {
            prev = value;
            continue;
        }

        push_segment(&mut segments, start, prev);
        start = value;
        prev = value;
    }

    push_segment(&mut segments, start, prev);
    segments
}

fn push_segment(segments: &mut Vec<ConsecutiveSegment>, start: u32, end: u32) {
    if start == end {
        segments.push(ConsecutiveSegment::Single(start));
    } else {
        segments.push(ConsecutiveSegment::Range { start, end });
    }
}
