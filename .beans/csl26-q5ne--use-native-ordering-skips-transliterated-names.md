---
# csl26-q5ne
title: use-native-ordering skips transliterated names
status: todo
type: bug
created_at: 2026-06-09T21:39:40Z
updated_at: 2026-06-09T21:39:40Z
---

use-native-ordering: true does not apply to transliterated names: the script-config lookup in crates/citum-engine/src/render/names.rs fires on the rendered Latin-script name rather than the Han/Hangul source name, so family-first ordering is not applied. CNE Chicago expects 'Hua Linfu' / 'Kang U-bang' / 'Abe Yoshio' (family first); engine renders 'Linfu Hua' etc. Repro: render tests/fixtures/multilingual/multilingual-cne-chicago.yaml with styles/embedded/chicago-notes-18th-cne.yaml. See TODO(bean:) in crates/citum-engine/tests/multilingual.rs.
