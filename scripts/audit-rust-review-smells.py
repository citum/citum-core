#!/usr/bin/env python3
"""Advisory audit for Rust test independence and string ownership review smells."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parents[1]
RUST_ROOTS = ("crates", "tests")

HOT_PATH_PARTS = (
    "processor",
    "render",
    "rendering",
    "bibliography",
    "citation",
    "values",
    "locale",
    "sorting",
    "disambiguation",
    "template_compiler",
)

SHORT_LITERAL = r'"(?:[^"\\]|\\.){0,24}"'


@dataclass(frozen=True)
class Finding:
    """A single advisory review finding."""

    severity: str
    category: str
    path_kind: str
    path: str
    line: int
    rule: str
    message: str
    snippet: str


@dataclass(frozen=True)
class Rule:
    """A line-oriented advisory rule."""

    name: str
    category: str
    severity: str
    pattern: re.Pattern[str]
    message: str
    include_kinds: tuple[str, ...] = ("prod", "hot-path", "test", "bench", "fixture")


RULES = (
    Rule(
        name="string-contains-literal-allocation",
        category="string-allocation",
        severity="high",
        pattern=re.compile(r"\.contains\(&" + SHORT_LITERAL + r"\.to_string\(\)\)"),
        message=(
            "Borrow the literal for membership checks instead of allocating a short "
            "String solely for comparison."
        ),
        include_kinds=("prod", "hot-path", "test"),
    ),
    Rule(
        name="push-str-format",
        category="string-allocation",
        severity="medium",
        pattern=re.compile(r"\.push_str\(&format!\("),
        message=(
            "Review whether write! on the existing String would avoid a temporary "
            "allocation; keep format! only when readability is worth it."
        ),
        include_kinds=("prod", "hot-path"),
    ),
    Rule(
        name="short-literal-fallback-allocation",
        category="string-allocation",
        severity="medium",
        pattern=re.compile(
            r"\.(?:ok_or(?:_else)?|unwrap_or(?:_else)?)\([^)]*"
            + SHORT_LITERAL
            + r"\.to_string\(\)"
        ),
        message=(
            "Short literal fallback allocates an owned String; confirm this is an "
            "ownership boundary or use a borrowed/static error shape."
        ),
        include_kinds=("prod", "hot-path"),
    ),
    Rule(
        name="ignored-test",
        category="test-independence",
        severity="high",
        pattern=re.compile(r"#\s*\[\s*ignore(?:\s*[=\]])"),
        message=(
            "Ignored tests need an explicit reason and tracking path; otherwise they "
            "can hide regressions."
        ),
        include_kinds=("test",),
    ),
    Rule(
        name="render-output-contains-assertion",
        category="test-independence",
        severity="high",
        pattern=re.compile(
            r'assert!\((?![^;]*err(?:or)?\b)[^;]*\.contains\(\s*"[^"]{0,29}"\s*\)'
        ),
        message=(
            "Short contains() assertions (< 30 chars) on rendered output are banned. "
            "Use assert_eq! with the full expected string instead. If a partial match "
            "is genuinely needed, the substring must be >= 30 chars and the test name "
            "must include '_contains_' or '_partial_'. See CODING_STANDARDS.md."
        ),
        include_kinds=("test",),
    ),

    Rule(
        name="expected-derived-from-actual",
        category="test-independence",
        severity="high",
        pattern=re.compile(
            r"(?:let\s+expected\s*=\s*(?:actual|result|rendered|output)|"
            r"let\s+actual\s*=\s*expected|"
            r"assert_eq!\(\s*(\w+)\s*,\s*\1\s*\))"
        ),
        message=(
            "Expected values must come from an independent fixture, oracle, spec, or "
            "literal behavior contract, not from the actual output under test."
        ),
        include_kinds=("test",),
    ),
)


def run_git(args: list[str]) -> str:
    """Run a git command in the repository and return stdout."""

    completed = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return completed.stdout


def tracked_rust_files() -> list[Path]:
    """Return all tracked Rust files under the configured Rust roots."""

    output = run_git(["ls-files", "--", *RUST_ROOTS])
    return sorted(ROOT / line for line in output.splitlines() if line.endswith(".rs"))


def changed_rust_files() -> list[Path]:
    """Return changed Rust files relative to the merge base with HEAD."""

    base = run_git(["merge-base", "HEAD", "origin/main"]).strip()
    output = run_git(
        [
            "diff",
            "--name-only",
            "--diff-filter=ACMR",
            f"{base}...HEAD",
            "--",
            *RUST_ROOTS,
        ]
    )
    return sorted(ROOT / line for line in output.splitlines() if line.endswith(".rs"))


def classify(path: Path) -> str:
    """Classify a Rust path by review surface."""

    rel = path.relative_to(ROOT)
    parts = rel.parts
    rel_text = rel.as_posix()

    if "/benches/" in f"/{rel_text}/":
        return "bench"
    if "/fixtures/" in f"/{rel_text}/":
        return "fixture"
    if (
        "/tests/" in f"/{rel_text}/"
        or path.name == "tests.rs"
        or path.name.endswith("_tests.rs")
        or path.stem.endswith("_test")
    ):
        return "test"
    if any(part in HOT_PATH_PARTS for part in parts):
        return "hot-path"
    return "prod"


def dense_literal_to_string_findings(path: Path, lines: list[str], path_kind: str) -> list[Finding]:
    """Flag dense clusters of short literal to_string calls outside test surfaces."""

    if path_kind not in {"prod", "hot-path"}:
        return []

    # Ignore match arms (ending in =>)
    literal_pattern = re.compile(r'(?<!=>\s)' + SHORT_LITERAL + r"\.to_string\(\)")
    test_like_pattern = re.compile(r"\b(?:assert|panic)!\(|#\s*\[\s*(?:test|rstest|case|cfg\(test\))")
    findings: list[Finding] = []
    window_size = 25
    threshold = 6 if path_kind == "hot-path" else 10

    for start in range(0, len(lines), window_size):
        window = lines[start : start + window_size]
        count = sum(
            len(literal_pattern.findall(line))
            for line in window
            if not test_like_pattern.search(line)
        )
        if count < threshold:
            continue
        first_snippet = next(
            (
                line.strip()
                for line in window
                if literal_pattern.search(line) and not test_like_pattern.search(line)
            ),
            "",
        )
        findings.append(
            Finding(
                severity="medium",
                category="string-allocation",
                path_kind=path_kind,
                path=path.relative_to(ROOT).as_posix(),
                line=start + 1,
                rule="dense-literal-to-string",
                message=(
                    f"{count} short literal `.to_string()` calls in {window_size} lines. "
                    "Review whether this is data construction, an ownership boundary, "
                    "or avoidable churn."
                ),
                snippet=first_snippet,
            )
        )
    return findings


def scan_file(path: Path) -> list[Finding]:
    """Scan one Rust file for advisory findings."""

    path_kind = classify(path)
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError:
        lines = path.read_text(errors="replace").splitlines()

    findings: list[Finding] = []
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        for rule in RULES:
            if path_kind not in rule.include_kinds:
                continue
            if not rule.pattern.search(line):
                continue
            findings.append(
                Finding(
                    severity=rule.severity,
                    category=rule.category,
                    path_kind=path_kind,
                    path=path.relative_to(ROOT).as_posix(),
                    line=line_number,
                    rule=rule.name,
                    message=rule.message,
                    snippet=stripped,
                )
            )

    findings.extend(dense_literal_to_string_findings(path, lines, path_kind))
    return findings


def changed_test_without_behavior_evidence(paths: Iterable[Path]) -> list[Finding]:
    """Flag changed Rust test files when the PR has no obvious behavior evidence changes."""

    changed = list(paths)
    test_files = [path for path in changed if classify(path) == "test"]
    if not test_files:
        return []

    evidence_paths = (
        "crates/",
        "tests/fixtures/",
        "styles/",
        "styles-legacy/",
        "scripts/",
        "docs/specs/",
        "docs/architecture/",
        "docs/policies/",
    )
    changed_names = [
        line
        for line in run_git(["diff", "--name-only", "origin/main...HEAD"]).splitlines()
        if line
    ]
    has_behavior_evidence = any(
        name.startswith(evidence_paths) and not name.endswith(".rs") for name in changed_names
    ) or any(classify(ROOT / name) in {"prod", "hot-path"} for name in changed_names if name.endswith(".rs"))

    if has_behavior_evidence:
        return []

    return [
        Finding(
            severity="medium",
            category="test-independence",
            path_kind="test",
            path=path.relative_to(ROOT).as_posix(),
            line=1,
            rule="changed-test-without-behavior-evidence",
            message=(
                "This PR changes a Rust test without an obvious production, fixture, "
                "oracle, or spec/doc evidence change. Confirm the test is not merely "
                "adapting expectations to current behavior."
            ),
            snippet="",
        )
        for path in test_files
    ]


def severity_rank(finding: Finding) -> tuple[int, str, int, str]:
    """Sort findings by severity and location."""

    ranks = {"high": 0, "medium": 1, "low": 2}
    return (ranks.get(finding.severity, 9), finding.path, finding.line, finding.rule)


def print_text(findings: list[Finding]) -> None:
    """Print findings grouped by severity and category."""

    if not findings:
        print("No advisory Rust review smells found.")
        return

    print(f"Advisory Rust review smells: {len(findings)} finding(s)")
    print("This script is informational; it does not define a merge gate.\n")

    for severity in ("high", "medium", "low"):
        severity_findings = [f for f in findings if f.severity == severity]
        if not severity_findings:
            continue
        print(f"{severity.upper()}")
        for finding in severity_findings:
            print(
                f"  {finding.path}:{finding.line} "
                f"[{finding.path_kind}/{finding.category}/{finding.rule}]"
            )
            print(f"    {finding.message}")
            if finding.snippet:
                print(f"    {finding.snippet}")
        print()


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments."""

    parser = argparse.ArgumentParser(
        description=(
            "Advisory Rust review-smell audit for test independence and string ownership."
        )
    )
    scope = parser.add_mutually_exclusive_group(required=True)
    scope.add_argument("--all", action="store_true", help="scan all tracked Rust files")
    scope.add_argument(
        "--changed",
        action="store_true",
        help="scan changed Rust files relative to origin/main",
    )
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    return parser.parse_args()


def main() -> int:
    """Run the advisory audit."""

    args = parse_args()
    files = tracked_rust_files() if args.all else changed_rust_files()
    findings = [finding for path in files for finding in scan_file(path)]
    if args.changed:
        findings.extend(changed_test_without_behavior_evidence(files))

    findings.sort(key=severity_rank)

    if args.json:
        print(
            json.dumps(
                {
                    "advisory": True,
                    "scope": "all" if args.all else "changed",
                    "finding_count": len(findings),
                    "findings": [asdict(finding) for finding in findings],
                },
                indent=2,
            )
        )
    else:
        print_text(findings)

    return 0


if __name__ == "__main__":
    sys.exit(main())
