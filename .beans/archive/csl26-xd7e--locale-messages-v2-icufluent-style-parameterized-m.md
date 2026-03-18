---
# csl26-xd7e
title: 'Locale Messages v2: ICU/Fluent-style parameterized message system'
status: completed
type: feature
priority: high
created_at: 2026-03-18T10:09:32Z
updated_at: 2026-03-18T14:22:19Z
---

Redesign Citum's localization layer from flat key→string YAML term files to an
ICU Message Format 1 (MF1) parameterized message system.

Spec: docs/specs/LOCALE_MESSAGES.md

## Goals
- LocalePreset: structured bundle of MF1 messages, dateFormats, numberFormats, grammarOptions, legacyTermAliases
- StylePreset × LocalePreset × LocaleOverride composition — no per-language style duplication
- Backwards-compatible: v1 locale files continue to work via legacy path

## Implementation Tasks

- [x] Phase 0: RawLocale/Locale schema additions (messages, dateFormats, grammarOptions, legacyTermAliases, localeSchemaVersion)
- [x] Phase 0: Dual-path lookup in general_term/locator_term/role_term
- [x] Phase 1: Convert high-impact terms in en-US.yaml and de-DE.yaml to ICU messages
- [x] Phase 1: scripts/migrate-locale-v1-to-v2.js automated converter
- [x] Phase 2: dateFormats map + grammarOptions expansion in locale YAML
- [x] Phase 2: Move punctuation_in_quote and related config to grammarOptions
- [x] Phase 3: LocaleOverride struct + Locale::apply_override() + locales/overrides/
- [x] Phase 3: options.localeOverride field in style YAML schema
- [x] Phase 4: citum locale lint subcommand (MF1 syntax validation + variable check)
- [x] Phase 4: citum style lint --locale cross-validation
- [x] Phase 4: MessageEvaluator trait + IcuMessageEvaluator (icu_plurals + MF1 interpreter)
- [x] Rendering benchmark regression check
