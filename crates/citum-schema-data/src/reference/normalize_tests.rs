/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Behaviour tests for genre/medium normalisation.

use super::InputReference;

fn norm(s: &str) -> String {
    InputReference::normalize_genre_medium(s)
}

#[test]
fn test_normalize_genre_medium() {
    assert_eq!(norm("Technical report"), "technical-report");
    assert_eq!(norm("PhD thesis"), "phd-thesis");
    assert_eq!(norm("Short film"), "short-film");
    assert_eq!(norm("video-interview"), "video-interview");
    assert_eq!(norm("film"), "film");
    assert_eq!(norm("Annual report"), "annual-report");
}
