---
# csl26-q5ne
title: use-native-ordering skips transliterated names
status: completed
type: bug
priority: normal
created_at: 2026-06-09T21:39:40Z
updated_at: 2026-06-09T22:02:24Z
---

use-native-ordering: true does not apply to transliterated names: the script-config lookup in crates/citum-engine/src/render/names.rs fires on the rendered Latin-script name rather than the Han/Hangul source name, so family-first ordering is not applied. CNE Chicago expects 'Hua Linfu' / 'Kang U-bang' / 'Abe Yoshio' (family first); engine renders 'Linfu Hua' etc. Repro: render tests/fixtures/multilingual/multilingual-cne-chicago.yaml with styles/embedded/chicago-notes-18th-cne.yaml. See TODO(bean:) in crates/citum-engine/tests/multilingual.rs.

## Summary of Changes

script_config_for_name (citum-engine/src/values/contributor/names.rs) now records the original_script chars of a transliterated multilingual name when detecting the script, so use-native-ordering fires on romanized CJK names (family-first 'Hua Linfu'). Also fixed a latent casing bug: style script keys like 'Han'/'Hangul' (ISO 15924 canonical casing) never matched the lowercase candidate keys; lookup is now case-insensitive. Verified by CNE tests in crates/citum-engine/tests/multilingual.rs.
