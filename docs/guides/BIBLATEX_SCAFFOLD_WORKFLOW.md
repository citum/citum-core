# Biblatex Scaffold Workflow

Use `scripts/scaffold-biblatex-style.js` when a Citum author wants a starter
YAML file for a biblatex-benchmarked style without pretending that biblatex
macro code has been converted.

The script reads rendered bibliography output from
`tests/snapshots/biblatex/*.json`, pairs it with the fixture used to produce
that snapshot, and emits a schema-valid Citum style scaffold. It is a
hand-finish aid only.

## Generate a Scaffold

For a style with an existing snapshot:

```bash
node scripts/scaffold-biblatex-style.js \
  --style numeric-comp \
  --output /tmp/numeric-comp-scaffold.yaml
```

Validate and smoke-render the result:

```bash
cargo run -q --bin citum -- check -s /tmp/numeric-comp-scaffold.yaml
cargo run -q --bin citum -- render refs \
  -b tests/fixtures/references-expanded.json \
  -s /tmp/numeric-comp-scaffold.yaml \
  --mode bib
```

For a biblatex style without a snapshot, either generate the snapshot first:

```bash
node scripts/gen-biblatex-snapshot.js --style nature
node scripts/scaffold-biblatex-style.js \
  --style nature \
  --output /tmp/nature-scaffold.yaml
```

or let the scaffold command generate it:

```bash
node scripts/scaffold-biblatex-style.js \
  --style nature \
  --generate-snapshot \
  --output /tmp/nature-scaffold.yaml
```

Use `--force-snapshot` when the snapshot should be refreshed even if it already
exists. `--bib`, `--biblatex-opts`, and `--cite` are forwarded to
`scripts/gen-biblatex-snapshot.js`.

## What It Infers

The scaffold script only uses evidence visible in rendered bibliography text:

- numeric versus author-date processing
- bibliography label punctuation
- basic order differences between fixture order and rendered order
- contributor name order, initials/full-name hints, delimiters, and conjunction
- title quote and casing hints
- year placement
- common bibliography component order
- bibliography entry suffix and component separator

These hints are enough to remove boilerplate from a hand-authored style. They
are not enough to finish fidelity work.

## What Must Be Finished by Hand

Rendered bibliography text cannot reveal biblatex macro logic. Authors must
complete or verify:

- citation-side behavior beyond a minimal numeric or author-date placeholder
- type variants and per-reference-type layout differences
- conditional fallback logic
- exact sorting rules
- edge cases not represented in the fixture
- fidelity against the biblatex snapshot and Citum render output

Keep the generated warning header in review diffs until the style is actually
hand-finished. Once the scaffold has been turned into a production style, remove
the warning and replace the generic description with a style-specific one.

## Scope Boundary

Do not add biblatex conversion logic to `crates/citum-migrate`. That crate is
for CSL 1.0 XML migration. Biblatex `.bbx` and `.cbx` files are LaTeX macro
programs, not a structurally analogous style tree.
