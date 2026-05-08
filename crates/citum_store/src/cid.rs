/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Content-addressed identifiers (CIDv1) for Citum styles.
//!
//! Citum styles are opaque byte sequences as far as content addressing is
//! concerned — the engine treats a style file as YAML-or-JSON-or-CBOR, and the
//! identity of a style is the identity of its bytes. This module wraps the
//! [`cid`] and [`multihash`] crates with a thin Citum-specific surface:
//!
//! - [`compute_style_cid`] hashes raw bytes as SHA-256 and returns the
//!   canonical CIDv1 string (codec `0x55` raw, multibase `b` base32 lower).
//! - [`verify_cid`] checks a `bafkrei…` CID against fresh bytes; on mismatch
//!   it returns a [`ResolverError::IntegrityFailure`] carrying both the
//!   expected and the actual CID.
//! - [`canonicalize_cid`] normalizes a CID string (with or without the
//!   `cid:` URI scheme) to its canonical lowercase base32 form.
//! - [`is_cid_uri`] / [`strip_cid_scheme`] handle the URI-scheme prefix.
//!
//! The chosen encoding mirrors the IPFS default for raw blocks, so a Citum
//! CID is interchangeable with any IPFS gateway that serves raw content.

#![cfg(feature = "http")]

use cid::Cid;
use multihash::Multihash;
use sha2::{Digest, Sha256};

use crate::resolver::ResolverError;

/// CIDv1 codec for opaque (raw) bytes.
///
/// See <https://github.com/multiformats/multicodec/blob/master/table.csv>.
const RAW_CODEC: u64 = 0x55;

/// Multihash code for SHA-256.
const SHA256_CODE: u64 = 0x12;

/// Compute the canonical CIDv1 for a Citum style payload.
///
/// Returns a string of the form `bafkrei…` — multibase `b` (base32 lower)
/// over CIDv1 (raw codec, SHA-256 hash). The function never fails for
/// well-formed inputs; the result is purely a function of the bytes.
///
/// # Panics
///
/// Panics if the multihash crate refuses to wrap a 32-byte SHA-256 digest,
/// which would indicate a programmer error in this module rather than a
/// runtime condition.
#[must_use]
#[allow(
    clippy::expect_used,
    reason = "Multihash::wrap on a 32-byte SHA-256 input is infallible by construction"
)]
pub fn compute_style_cid(bytes: &[u8]) -> String {
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    let mh = Multihash::<64>::wrap(SHA256_CODE, &digest)
        .expect("32-byte SHA-256 digest always fits in Multihash<64>");
    let cid = Cid::new_v1(RAW_CODEC, mh);
    cid.to_string()
}

/// Verify that `bytes` hashes to the canonical content of `expected_cid`.
///
/// On match, returns `Ok(())`. On mismatch, returns
/// [`ResolverError::IntegrityFailure`] populated with both the
/// canonicalized `expected` CID and the `actual` CID computed from `bytes`,
/// so callers can surface the divergence.
///
/// # Errors
///
/// Returns [`ResolverError::IntegrityFailure`] when the CIDs differ.
/// Returns [`ResolverError::InvalidStyle`] when `expected_cid` is not a
/// parseable CIDv1 string.
pub fn verify_cid(uri: &str, expected_cid: &str, bytes: &[u8]) -> Result<(), ResolverError> {
    let actual = compute_style_cid(bytes);
    let expected_canonical = canonicalize_cid(expected_cid)?;
    if actual == expected_canonical {
        Ok(())
    } else {
        Err(ResolverError::IntegrityFailure {
            uri: uri.to_string(),
            expected: expected_canonical,
            actual,
        })
    }
}

/// Parse a CIDv1 string and return its canonical lowercase base32 form.
///
/// This normalizes equivalent CIDs that differ only in case or multibase
/// representation. Returns the input string unchanged when it already parses
/// as a canonical CID.
///
/// # Errors
///
/// Returns [`ResolverError::InvalidStyle`] when the CID is unparseable.
pub fn canonicalize_cid(s: &str) -> Result<String, ResolverError> {
    let trimmed = s.strip_prefix("cid:").unwrap_or(s);
    let cid: Cid = trimmed.parse().map_err(|err: cid::Error| {
        ResolverError::InvalidStyle(format!("invalid CID '{s}': {err}").into())
    })?;
    Ok(cid.to_string())
}

/// Strip a leading `cid:` URI scheme and return the bare CID string.
///
/// Returns the input unchanged when no `cid:` prefix is present.
#[must_use]
pub fn strip_cid_scheme(uri: &str) -> &str {
    uri.strip_prefix("cid:").unwrap_or(uri)
}

/// Returns true when `uri` is a `cid:` content-addressed reference.
#[must_use]
pub fn is_cid_uri(uri: &str) -> bool {
    uri.starts_with("cid:")
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;

    /// Canonical CIDv1 (raw codec 0x55, sha-256, multibase base32 lowercase)
    /// for the bytes of `b"hello world"`. Matches `ipfs add --raw-leaves`
    /// output for the same input.
    const HELLO_WORLD_CID: &str = "bafkreifzjut3te2nhyekklss27nh3k72ysco7y32koao5eei66wof36n5e";

    #[test]
    fn compute_cid_is_stable_for_known_input() {
        let cid = compute_style_cid(b"hello world");
        assert_eq!(cid, HELLO_WORLD_CID, "canonical CIDv1 raw/sha256");
    }

    #[test]
    fn compute_cid_changes_when_bytes_change() {
        let a = compute_style_cid(b"alpha");
        let b = compute_style_cid(b"beta");
        assert_ne!(a, b);
    }

    #[test]
    fn verify_cid_accepts_matching_bytes() {
        let bytes = b"some style content";
        let cid = compute_style_cid(bytes);
        assert!(verify_cid("test://uri", &cid, bytes).is_ok());
    }

    #[test]
    fn verify_cid_accepts_cid_scheme_prefix() {
        let bytes = b"some style content";
        let cid = compute_style_cid(bytes);
        let scheme = format!("cid:{cid}");
        assert!(verify_cid("test://uri", &scheme, bytes).is_ok());
    }

    #[test]
    fn verify_cid_rejects_tampered_bytes() {
        let original = b"trusted content";
        let cid = compute_style_cid(original);
        let tampered = b"untrusted content";
        let err = verify_cid("test://uri", &cid, tampered).expect_err("must reject");
        match err {
            ResolverError::IntegrityFailure {
                expected, actual, ..
            } => {
                assert_eq!(expected, cid);
                assert_ne!(expected, actual);
            }
            other => panic!("expected IntegrityFailure, got {other:?}"),
        }
    }

    #[test]
    fn verify_cid_rejects_invalid_cid_string() {
        let err = verify_cid("test://uri", "not-a-cid", b"x").expect_err("must reject");
        assert!(matches!(err, ResolverError::InvalidStyle(_)));
    }

    #[test]
    fn is_cid_uri_detects_scheme() {
        assert!(is_cid_uri("cid:bafkreiabc"));
        assert!(!is_cid_uri("https://example.org/x.yaml"));
        assert!(!is_cid_uri("bafkreiabc"));
    }

    #[test]
    fn strip_cid_scheme_handles_both_forms() {
        assert_eq!(strip_cid_scheme("cid:bafkreiabc"), "bafkreiabc");
        assert_eq!(strip_cid_scheme("bafkreiabc"), "bafkreiabc");
    }
}
