#!/usr/bin/env python3
"""Regression tests for the advisory Rust review-smell audit.

Run directly: ``python3 scripts/test_audit_rust_review_smells.py``.
Guards the precision/recall fixes: assert-flavour exclusion in the dense
literal heuristic and statement-level detection of multi-line assertions.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "audit_rust_review_smells",
    Path(__file__).resolve().parent / "audit-rust-review-smells.py",
)
audit = importlib.util.module_from_spec(_SPEC)
assert _SPEC and _SPEC.loader
# Register before exec so dataclass annotation resolution can find the module.
sys.modules["audit_rust_review_smells"] = audit
_SPEC.loader.exec_module(audit)


def rule(name: str):
    """Return the rule object with the given name."""

    return next(r for r in audit.RULES if r.name == name)


def test_iter_statements_joins_multiline_and_reports_start() -> None:
    """A `;`-terminated statement spanning lines is one chunk at its start line."""

    lines = ["assert!(", '    out.contains("x"),', '    "msg"', ");"]
    statements = list(audit.iter_statements(lines))
    assert statements == [(1, 'assert!(     out.contains("x"),     "msg" );')]


def test_render_rule_matches_multiline_short_contains() -> None:
    """The render rule fires on a multi-line short contains() once joined."""

    pattern = rule("render-output-contains-assertion").pattern
    joined = 'assert!( rendered.contains("1919"), "year must appear" );'
    assert pattern.search(joined)


def test_render_rule_ignores_long_substring() -> None:
    """A contains() substring of 30+ chars is exempt and must not match."""

    pattern = rule("render-output-contains-assertion").pattern
    long_literal = "x" * 35
    joined = f'assert!( rendered.contains("{long_literal}") );'
    assert not pattern.search(joined)


def test_dense_excludes_assert_eq_literals() -> None:
    """assert_eq!/assert_ne! data literals are test data, not production churn."""

    lines = [f'        assert_eq!(f(n), Some("{c}".to_string()));' for c in "abcdefgh"]
    path = audit.ROOT / "crates" / "fake" / "src" / "values.rs"
    findings = audit.dense_literal_to_string_findings(path, lines, "hot-path")
    assert findings == []


def test_dense_skips_trailing_cfg_test_module() -> None:
    """Literals inside a trailing #[cfg(test)] module are not counted."""

    lines = ["#[cfg(test)]", "mod tests {"]
    lines += [f'    let v{i} = "x{i}".to_string();' for i in range(8)]
    lines += ["}"]
    path = audit.ROOT / "crates" / "fake" / "src" / "values.rs"
    findings = audit.dense_literal_to_string_findings(path, lines, "hot-path")
    assert findings == []


def test_dense_flags_real_production_churn() -> None:
    """Dense short literal to_string churn in production code still flags."""

    lines = [f'    map.insert(k{i}, "v{i}".to_string());' for i in range(8)]
    path = audit.ROOT / "crates" / "fake" / "src" / "values.rs"
    findings = audit.dense_literal_to_string_findings(path, lines, "hot-path")
    assert len(findings) == 1
    assert findings[0].rule == "dense-literal-to-string"


def test_ignored_test_flags_only_bare_ignore() -> None:
    """ignored-test flags bare #[ignore] but not a reasoned one."""

    pattern = rule("ignored-test").pattern
    assert pattern.search("    #[ignore]")
    assert not pattern.search('    #[ignore = "flaky, see #123"]')


def main() -> int:
    """Run every test function defined in this module."""

    tests = sorted(
        (name, fn)
        for name, fn in globals().items()
        if name.startswith("test_") and callable(fn)
    )
    for name, fn in tests:
        fn()
        print(f"ok  {name}")
    print(f"\n{len(tests)} passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
