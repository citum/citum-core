---
# csl26-qrpo
title: Swap mf2_parser for ICU4X icu_message_format when stable
status: todo
type: task
priority: deferred
tags:
    - locale
created_at: 2026-03-22T12:45:31Z
updated_at: 2026-04-25T20:20:06Z
---

The current Mf2MessageEvaluator in crates/citum-schema-style/src/locale/message.rs is a
custom dependency-free implementation (no external crate — mf2_parser was GPL-3.0
incompatible). When icu_message_format (ICU4X) reaches stable:
1. Add icu_message_format dep
2. Implement IcuMf2MessageEvaluator (same MessageEvaluator trait)
3. Swap the evaluator in default_evaluator() — one line change

No locale files or call sites change. MF2 syntax is identical between implementations.

Blocked by: ICU4X icu_message_format reaching stable release.
Tracking issue: https://github.com/unicode-org/icu4x/issues/3028

As of 2026-04-25, unicode-org/icu4x#7884 is an open in-progress MF2
implementation PR. Treat it as useful upstream signal, not an available
dependency for Citum branch work.
