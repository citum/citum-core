/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Shared helpers for collapsing ordered consecutive numbering into spans.

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
#[must_use]
pub fn consecutive_segments(values: &[u32]) -> Vec<ConsecutiveSegment> {
    let mut iter = values.iter();
    let Some(&first) = iter.next() else {
        return Vec::new();
    };

    let mut segments = Vec::new();
    let mut start = first;
    let mut prev = first;

    for &value in iter {
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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;

    #[test]
    fn test_consecutive_segments() {
        for (input, expected) in [
            (&[][..], vec![]),
            (&[1][..], vec![ConsecutiveSegment::Single(1)]),
            (
                &[1, 2, 3][..],
                vec![ConsecutiveSegment::Range { start: 1, end: 3 }],
            ),
            (
                &[1, 3, 5][..],
                vec![
                    ConsecutiveSegment::Single(1),
                    ConsecutiveSegment::Single(3),
                    ConsecutiveSegment::Single(5),
                ],
            ),
            (
                &[1, 2, 4, 5, 6, 8][..],
                vec![
                    ConsecutiveSegment::Range { start: 1, end: 2 },
                    ConsecutiveSegment::Range { start: 4, end: 6 },
                    ConsecutiveSegment::Single(8),
                ],
            ),
            (
                &[1, 1, 2, 2, 3][..],
                vec![ConsecutiveSegment::Range { start: 1, end: 3 }],
            ),
            (
                &[3, 2, 1][..],
                vec![
                    ConsecutiveSegment::Single(3),
                    ConsecutiveSegment::Single(2),
                    ConsecutiveSegment::Single(1),
                ],
            ),
        ] {
            assert_eq!(consecutive_segments(input), expected);
        }
    }
}
