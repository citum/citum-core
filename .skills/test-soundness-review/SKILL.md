---
name: test-soundness-review
description: >
  Repo-owned test soundness auditor and navigator for citum-core. Reasons from
  a spec (not the implementation) to classify each test as good, suspicious,
  broken, or redundant; simultaneously reviews the spec for ambiguity,
  contradiction, and silence, halting to prompt the user when a spec defect
  blocks an honest verdict. Records state in the living ledger
  docs/architecture/TEST_SOUNDNESS_STATUS.md plus a dated audit record — never
  ephemeral JSON.

  Trigger on: "/test-soundness-review", "review my tests against the spec",
  "audit test soundness", "are my tests vacuous", "find broken tests", "find
  redundant tests", "are these tests just for the sake of it", "test quality
  review", "review the spec while you're at it", "what's the state of our test
  quality", "what test audits are outstanding", "what's next for test
  soundness". Also trigger proactively after a disambiguation, sorting,
  algorithm, or engine feature is implemented when the user asks "do the tests
  actually cover the spec?". Takes up to two arguments:
  /test-soundness-review [spec-path] [test-file-or-glob]
---

# test-soundness-review

Audit test functions against the spec that governs them, and keep a durable
record of the state of test quality across `citum-core`. The goal is to find
tests that could pass while the implementation still violates the spec — and
tests that exist for their own sake and prove nothing — *not* to review code
style.

The verdict bar is the shared **"What makes a test worth keeping"** contract in
[`docs/guides/CODING_STANDARDS.md`](../../docs/guides/CODING_STANDARDS.md). That
section is the source of truth; this skill operationalises it. If this skill and
that contract ever disagree, the contract wins — fix the skill.

## Arguments

```
/test-soundness-review [spec-path] [test-file-or-glob]
```

Examples:
```
/test-soundness-review docs/specs/SORTING.md crates/citum-engine/tests/
/test-soundness-review docs/specs/DISAMBIGUATION.md crates/citum-engine/tests/citations.rs
/test-soundness-review            # no args — navigate the ledger, propose what's next
```

Both arguments are optional. With **no arguments**, run Step 0 only and stop —
the skill becomes a way to navigate test-quality state. With **one argument**,
ask for the missing one before proceeding.

---

## Step 0 — Read the ledger first (always)

Read [`docs/architecture/TEST_SOUNDNESS_STATUS.md`](../../docs/architecture/TEST_SOUNDNESS_STATUS.md)
before anything else. It is the index of what has been audited, addressed, and
what remains `todo`.

- **No spec named** → summarise the ledger (counts by status, any `needs-rework`
  rows blocked on a spec decision) and offer the next few `todo` rows ranked by
  risk. Stop there; let the user pick.
- **Spec named** → find its row (or note that you'll create one). If the row is
  already `addressed`, say so and ask whether to re-audit before spending effort.

The ledger is also where you'll write back at the end (Step 6). Treat it as the
entry and exit point of every run.

## Step 1 — Read the spec and build the behaviour map

Read `<spec-path>` in full. Extract:
- Documented behaviours (numbered sections, rules, invariants).
- Acceptance criteria (checklist items).
- Strategy cascades / decision trees.
- Explicit "must", "must not", "is never consulted", "always" statements.

Build a map: **spec section → observable behaviour**. You'll use it both to
judge each test and to drive the spec review in Step 2.

## Step 2 — Review the spec itself (first-class, runs alongside test reading)

A test audit is only as trustworthy as the spec it reasons from. As you build
the behaviour map, record spec defects in three buckets:

- **Ambiguity** — a rule that admits two readings, where a test could be "right"
  under either. (Most dangerous: it lets a test launder the ambiguity into a
  false `good`.)
- **Contradiction** — two sections that cannot both hold.
- **Silence** — a behaviour a test asserts, or a fixture clearly implies, that
  the spec never states.

For each defect, classify its **audit impact**:

- `blocking` — you cannot honestly classify a test without the user first
  deciding what the spec means.
- `advisory` — worth fixing, but doesn't change any verdict.

**Gating.** If any defect is `blocking`, **stop and prompt the user** before
finalising verdicts. Quote the spec section, name the affected tests, and give
2–3 concrete resolution options. Do not pick a reading yourself: choosing one
silently is exactly the failure this skill exists to catch, and per the repo
rule you "never make content decisions unilaterally." Advisory defects are
recorded (Step 6) and summarised, but don't halt the run.

## Step 3 — Enumerate and read the tests

Enumerate test functions in the glob. Prefer jcodemunch (`get_file_outline`,
`get_symbol`) over raw reads — it's the repo's standard and token-efficient.
Fallback:
```bash
grep -nE "fn |#\[test\]|#\[rstest\]" <file>
```
Also collect any `announce_behavior(...)` text or docstrings — a second source
of "what the author thought this checks".

For each test, read the body: fixture setup, the call under test, and **every**
assertion. Read shared helpers it calls (e.g. `run_test_case_native`,
`build_author_date_style`) — the helper's defaults change what the test proves.

## Step 4 — Classify each test

Reason from the **spec**, not the implementation. The question is always:
*could this test pass while the implementation still violates the spec section
it claims to cover?* — and *does this test make an independent claim at all?*

### Verdicts

**`broken`** — passes but proves nothing about the spec:
- `contains()` / `!contains()` on rendered output under ~30 chars (a garbled
  renderer can still hit a short substring). Note: the repo bans `contains()` on
  rendered output outright unless the substring is ≥30 chars and the test name
  signals it — see CODING_STANDARDS §Test Independence.
- No assertion at all (vacuous).
- Asserts an intermediate value but never the final rendered output.
- A stale capture stub left in (`panic!("intentional capture")`).

**`suspicious`** — may be correct but needs investigation:
- Name / `announce_behavior` describes behaviour X, but the fixture can never
  trigger X (e.g. "given-name expansion applied" where every colliding ref has a
  different year, so no collision group ever forms).
- Expected output looks **invented** rather than captured (round numbers,
  suspiciously clean delimiters that don't match the renderer).
- `announce_behavior` claims "subsequent form" / "repeat citation" but the test
  issues one batch with no `Position::Subsequent`.
- Counts occurrences (`matches(...).count()`) without verifying location.

**`redundant`** — passes, may even be correct, but adds **no independent claim**
(over-testing; see the shared contract). Recommend **delete or merge into the
canonical sibling**, not rewrite — fewer sharp tests beat many overlapping ones,
and deletion is an encouraged outcome here:
- Near-duplicate of an existing test on the same fixture + same assertion
  dimension (no new type, field shape, position, or edge condition).
- Tautological / self-evidently true: `assert!(true)`, asserting a literal you
  just constructed, round-tripping a value through no transformation.
- Tests language/library behaviour rather than Citum behaviour.
- Coverage theatre — exists only to touch a line, no observable-behaviour claim.

**`good`** — exact `assert_eq!` on full rendered output, fixture genuinely
triggers the documented behaviour, name matches what is tested, and it covers a
dimension no sibling already covers.

### Coverage gaps

For each spec section with documented behaviour but no corresponding test,
record a gap with a concrete recommendation (the fixture shape + assertion to
add). Cross-check the spec's acceptance criteria for premature `[x]`: if a
criterion is marked done but the function it depends on doesn't exist, record it
as a gap and recommend reverting the `[x]`.

## Step 5 — Execute follow-on actions: Fix → Trim → Add → Persist

After presenting the per-test table and coverage gaps, **proceed immediately** to
Fix → Trim → Add → Persist in that order. State the phase and what you are about
to do before each one; the user can say "stop" or "skip <phase>" to redirect.
The default is to do everything — do not stop to ask permission unless a decision
is genuinely unresolvable (ambiguous spec, missing fixture data, failing gate).

1. **Fix** — realign suspicious tests; rewrite broken assertions to exact
   `assert_eq!`. For each test marked **broken** or **suspicious**: if fixing
   requires pinning an expected string, use the capture-and-pin workflow — add a
   transient `eprintln!` + forced `panic!("capture")`, run just that test, read
   stderr/stdout, remove the stub, pin the real string. **Never invent expected
   values.** Run `cargo nextest run` after the Fix batch.
2. **Trim** — delete or merge `redundant` tests. For each deletion: confirm the
   surviving sibling still covers the dimension (grep for the spec section; verify
   at least one `good` test covers it) before deleting. Run `cargo nextest run`
   after the Trim batch.
3. **Add** — add tests for coverage gaps, capturing actual output first (same
   capture-and-pin workflow). Prioritise: (1) gaps that would catch a realistic
   regression, (2) gaps flagged as a spec-silence needing resolution, (3) the rest.
   Run `cargo nextest run` after the Add batch.
4. **Persist** — Step 6 (always, even if Fix/Trim/Add were skipped).

## Step 6 — Persist state (replaces the old JSON dump)

Two durable artifacts, both Markdown:

1. **Audit record** — write/refresh
   `docs/architecture/audits/YYYY-MM-DD_<TOPIC>_TEST_SOUNDNESS.md` (today's date;
   match the house style of existing records such as
   `2026-05-07_SQI_INTEGRITY_AUDIT.md`). Include:
   - Header: date, scope (spec + reviewed files), related docs.
   - A per-test table: `| Test | Location | Spec ref | Intended behaviour | What it does | Verdict | Action |`.
   - A **Spec Issues** section: every ambiguity/contradiction/silence with its
     `blocking`/`advisory` impact and recommendation.
   - A **Coverage Gaps** section.
   - A summary line: good / suspicious / broken / redundant counts.

2. **Ledger row** — upsert the spec's row in
   `docs/architecture/TEST_SOUNDNESS_STATUS.md`:
   `| Spec / Module | Last reviewed | Tests (G/S/B/R) | Open spec issues | Status | Audit record |`
   - `Last reviewed`: today.
   - `Tests (G/S/B/R)`: the four counts.
   - `Open spec issues`: refs to unresolved spec defects, or `—`.
   - `Status`: `todo` (never audited) · `audited` (reviewed, findings open) ·
     `addressed` (findings fixed) · `needs-rework` (blocked on a spec decision).
   - `Audit record`: link to the file from artifact 1.
   - Bump the ledger's "Last updated" banner.

The audit record is the detail; the ledger row is the index that points to it.
An agent resuming work greps the ledger for `todo` / `needs-rework`.

**Commit guidance:** describe what changed, not just the meta-status. Split
into two commits when both test and doc/spec files change:
- `test(engine): <what changed>` — test files only; subject names the action
  (e.g. "trim 3 vacuous sort tests", "add citation-sort gap tests").
- `docs(spec): <what changed>` — spec, ledger, skill files; subject names the
  outcome (e.g. "clarify two SORTING spec silences"). Body 3–5 lines max.

---

## Key rules

- **Reason from the spec, not the code.** If the spec says "year-suffix
  collision key uses only `issued` year", check whether the test would catch a
  regression to `original-date` — not whether the code currently does the right
  thing.
- **Capture, don't invent.** Invented expected strings are the primary source of
  `suspicious`. Always capture real output before pinning.
- **Redundant is a defect, not a freebie.** A test that adds no independent claim
  costs maintenance and dilutes signal. Trimming is success.
- **Blocking spec defects stop the run.** Don't guess a reading; prompt the user.
- **Pre-commit gate when you touch `.rs`:**
  `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run`.
  Skill/docs-only edits (`.md`) skip the Rust gate but should pass
  `./scripts/validate-frontmatter.sh --copilot-strict`.

## Related

- Shared verdict bar: [`docs/guides/CODING_STANDARDS.md`](../../docs/guides/CODING_STANDARDS.md) § "What makes a test worth keeping" and § "Test Independence".
- Coverage / fixture-shape domain knowledge: [`test-coverage`](../../.claude/skills/test-coverage/SKILL.md).
- Strategy context: [`docs/guides/TEST_STRATEGY.md`](../../docs/guides/TEST_STRATEGY.md).
- State ledger: [`docs/architecture/TEST_SOUNDNESS_STATUS.md`](../../docs/architecture/TEST_SOUNDNESS_STATUS.md).
