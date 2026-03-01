# Test Strategy: Oracle vs Citum-Native

## Current Approach: Oracle Tests

**Purpose**: Validate CSL 1.0 backward compatibility

**Method**:
- Test fixtures use CSL JSON format (what citeproc-js expects)
- Compare Citum output against citeproc-js (the oracle)
- Located: `tests/fixtures/references-expanded.json`
- Test harness: `../../../scripts/oracle-e2e.js`

**Limitations**:
- Cannot test Citum-specific features beyond CSL 1.0
- Constrained by CSL JSON schema

## Integration Test Organization

Integration tests for the `citum_engine` crate are organized into functional targets to allow for focused testing:

- **citations**: Citation rendering, disambiguation, and group logic.
- **bibliography**: Bibliography rendering, sorting, and author substitution.
- **metadata**: Name parsing, contributor extraction, and date normalization.
- **i18n**: Locale-specific terms, date formatting, and translation resolution.
- **document**: Full document processing and semantic output.

To run a specific target:
```bash
cargo nextest run --test citations
```

## Future: Citum-Native Tests

**Purpose**: Test features that go beyond CSL 1.0

**Features requiring native format**:
1. **Title/subtitle separation** - CSL 1.0 treats as single string
2. **EDTF dates** - CSL JSON uses simple date-parts arrays
3. **Scoped multilingual fields** - CSL 1.0 doesn't support (csln#66)
4. **Enhanced citation model** - mode, locator types (from citum_schema)
5. **Math in variables** - Need proper encoding (csln#64)
6. **Structured name particles** - More nuanced than CSL JSON

**Implementation plan** (deferred):
1. Create `tests/fixtures/csln-native-references.yaml` (or .json - serde supports both)
2. Build separate test harness (no oracle comparison - we ARE the reference)
3. Test Citum-specific rendering against expected outputs
4. Document intentional divergences from CSL 1.0

**Current intake policy**:
- Use the full local CSL suite as the candidate corpus.
- Treat external processor reports such as Hayagriva issue `#327` as one signal,
  not the definition of scope.
- Prioritize native intake by Citum relevance, oracle/style risk, current native
  coverage gaps, and renderer sensitivity.
- See `docs/architecture/CSL_NATIVE_TEST_INTAKE_ONE_SHOT_PR_2026-03-01.md` and
  `tests/fixtures/csl-native-intake-wave1.json`.

## Two-Phase Testing Strategy

### Phase 1: CSL 1.0 Parity (Current)
- Expand oracle test coverage to 15+ reference types
- Achieve high fidelity for tier 1 styles (Chicago, APA, Elsevier)
- Use CSL JSON format throughout
- **Goal**: Prove migration works for existing styles

### Phase 2: Citum Extensions (Future)
- Add Citum-native test fixtures
- Test features beyond CSL 1.0
- No oracle comparison (we define the behavior)
- **Goal**: Validate new capabilities

## Decision: Phase 1 First

We're deferring Citum-native tests to focus on CSL 1.0 parity. This is the right prioritization because:
- Need to prove the migration approach works
- Most styles will initially be migrated from CSL 1.0
- New features can be tested incrementally as they're added
- Keeps current work focused and achievable

## Related
- csln#64 - Math in variables
- csln#66 - Multilingual support
- Citation model in citum_schema (mode, locator types)
