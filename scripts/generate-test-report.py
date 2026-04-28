#!/usr/bin/env python3
"""
Generate human-oriented test behavior reports from nextest JUnit output.
"""

from __future__ import annotations

import argparse
import datetime as dt
import html
import re
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path


STATUS_ORDER = {"failed": 0, "passed": 1, "skipped": 2}
TITLE_OVERRIDES = {
    "i18n": "I18n",
}
LOWERCASE_TITLE_WORDS = {"a", "an", "and", "for", "of", "or", "the", "to"}
DEFAULT_REPORT_TITLE = "Engine Behavior Coverage"
DEFAULT_REPORT_LEDE = (
    "This page is generated from selected engine behavior suites. "
    "It is meant for human review, not for detailed CI diagnostics."
)
PILOT_SOURCES = {
    "bibliography": Path("crates/citum-engine/tests/bibliography.rs"),
    "citations": Path("crates/citum-engine/tests/citations.rs"),
    "document": Path("crates/citum-engine/tests/document.rs"),
    "i18n": Path("crates/citum-engine/tests/i18n.rs"),
    "metadata": Path("crates/citum-engine/tests/metadata.rs"),
    "multilingual": Path("crates/citum-engine/tests/multilingual.rs"),
    "sort_oracle": Path("crates/citum-engine/tests/sort_oracle.rs"),
}


@dataclass
class Scenario:
    domain: str
    family: str
    binary_name: str
    test_name: str
    original_id: str
    scenario: str
    status: str
    derived_from_name: bool
    source_file: Path | None
    source_function: str | None
    source_start_line: int | None
    source_end_line: int | None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate Markdown and HTML behavior reports from nextest JUnit XML."
    )
    parser.add_argument(
        "--junit",
        default="target/nextest/report/junit.xml",
        help="Path to the nextest JUnit XML file.",
    )
    parser.add_argument(
        "--output",
        default="target/test-report.md",
        help="Path to the Markdown report to write.",
    )
    parser.add_argument(
        "--output-html",
        default=None,
        help="Optional path to the HTML report to write.",
    )
    parser.add_argument(
        "--source-root",
        default=".",
        help="Repository root used to resolve source file paths.",
    )
    parser.add_argument(
        "--report-title",
        default=DEFAULT_REPORT_TITLE,
        help="Human-facing title for the report.",
    )
    parser.add_argument(
        "--report-lede",
        default=DEFAULT_REPORT_LEDE,
        help="Introductory sentence shown near the top of the report.",
    )
    parser.add_argument(
        "--source-map",
        action="append",
        default=[],
        metavar="BINARY=PATH",
        help="Additional binary-to-source mapping for source references.",
    )
    return parser.parse_args()


def strip_tag(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def titleize(segment: str) -> str:
    if segment in TITLE_OVERRIDES:
        return TITLE_OVERRIDES[segment]
    words = [word for word in re.split(r"[_-]+", segment) if word]
    titled: list[str] = []
    for index, word in enumerate(words):
        if index > 0 and word in LOWERCASE_TITLE_WORDS:
            titled.append(word)
        else:
            titled.append(word.capitalize())
    return " ".join(titled)


def snake_to_sentence(name: str) -> str:
    cleaned = re.sub(r"^case_\d+_", "", name)
    cleaned = cleaned.replace("_", " ").strip()
    return cleaned if cleaned else name


def extract_behavior(output: str | None) -> str | None:
    if not output:
        return None
    for line in output.splitlines():
        stripped = line.strip()
        if stripped.startswith("behavior:"):
            return stripped.removeprefix("behavior:").strip()
    return None


def derive_domain(binary_name: str, test_name: str) -> str:
    test_parts = [part for part in test_name.split("::") if part]
    base = titleize(binary_name) if binary_name else "Tests"
    raw_module_parts = test_parts[:-1]
    if raw_module_parts == ["tests"]:
        return base
    if len(raw_module_parts) == 1 and (
        raw_module_parts[0].startswith(("given_", "when_", "then_"))
        or "_when_" in raw_module_parts[0]
        or "_then_" in raw_module_parts[0]
    ):
        return base
    module_parts = [titleize(part) for part in raw_module_parts]
    if not module_parts:
        return base
    return f"{base} - {' / '.join(module_parts)}"


def summarize_counts(entries: list[Scenario]) -> str:
    failed = sum(1 for entry in entries if entry.status == "failed")
    skipped = sum(1 for entry in entries if entry.status == "skipped")
    scenario_label = "scenario" if len(entries) == 1 else "scenarios"

    if failed == 0 and skipped == 0:
        return f"{len(entries)} {scenario_label}."

    parts = [f"{len(entries)} {scenario_label}"]
    if failed:
        parts.append(f"{failed} failed")
    if skipped:
        parts.append(f"{skipped} skipped")
    return ", ".join(parts) + "."


def compute_status_counts(scenarios: list[Scenario]) -> dict[str, int]:
    counts = {"passed": 0, "failed": 0, "skipped": 0}
    for scenario in scenarios:
        counts[scenario.status] += 1
    return counts


def summarize_status_counts(counts: dict[str, int]) -> str:
    parts = [f"{counts['passed']} passed"]
    if counts["failed"]:
        parts.append(f"{counts['failed']} failed")
    if counts["skipped"]:
        parts.append(f"{counts['skipped']} skipped")
    return ", ".join(parts)


def slugify_heading(value: str) -> str:
    slug = value.strip().lower()
    slug = re.sub(r"[^\w\s-]", "", slug)
    slug = re.sub(r"[-\s]+", "-", slug)
    return slug.strip("-") or "section"


def parse_status(testcase: ET.Element) -> str:
    child_tags = {strip_tag(child.tag) for child in testcase}
    if "failure" in child_tags or "error" in child_tags:
        return "failed"
    if "skipped" in child_tags:
        return "skipped"
    return "passed"


def derive_source_function(test_name: str) -> str | None:
    parts = [part for part in test_name.split("::") if part]
    if not parts:
        return None
    last = parts[-1]
    if last.startswith("case_") and len(parts) >= 2:
        return parts[-2]
    return last


def scan_source_locations(source_path: Path) -> dict[str, tuple[int, int]]:
    lines = source_path.read_text(encoding="utf-8").splitlines()
    pattern = re.compile(r"^\s*fn\s+([A-Za-z0-9_]+)\s*\(")
    locations: dict[str, tuple[int, int]] = {}
    index = 0

    while index < len(lines):
        match = pattern.match(lines[index])
        if not match:
            index += 1
            continue

        name = match.group(1)
        start = index + 1
        end = start
        brace_depth = 0
        saw_open_brace = False
        cursor = index

        while cursor < len(lines):
            line = lines[cursor]
            open_count = line.count("{")
            close_count = line.count("}")
            if open_count:
                saw_open_brace = True
            brace_depth += open_count - close_count
            end = cursor + 1
            cursor += 1
            if saw_open_brace and brace_depth <= 0:
                break

        locations[name] = (start, end)
        index = cursor

    return locations


def resolve_source_locations(
    scenarios: list[Scenario], source_root: Path
) -> None:
    caches: dict[Path, dict[str, tuple[int, int]]] = {}

    for scenario in scenarios:
        if scenario.source_file is None or scenario.source_function is None:
            continue

        if scenario.source_file not in caches:
            source_path = source_root / scenario.source_file
            if not source_path.exists():
                caches[scenario.source_file] = {}
            else:
                caches[scenario.source_file] = scan_source_locations(source_path)

        location = caches[scenario.source_file].get(scenario.source_function)
        if location is None:
            continue

        scenario.source_start_line, scenario.source_end_line = location


def format_source_reference(scenario: Scenario) -> str | None:
    if scenario.source_file is None or scenario.source_start_line is None:
        return None
    if scenario.source_end_line and scenario.source_end_line != scenario.source_start_line:
        return (
            f"{scenario.source_file}:{scenario.source_start_line}"
            f"-{scenario.source_end_line}"
        )
    return f"{scenario.source_file}:{scenario.source_start_line}"


def parse_source_maps(entries: list[str]) -> dict[str, Path]:
    source_map = dict(PILOT_SOURCES)
    for entry in entries:
        if "=" not in entry:
            raise ValueError(f"invalid --source-map value: {entry!r}")
        binary_name, path = entry.split("=", 1)
        source_map[binary_name.strip()] = Path(path.strip())
    return source_map


def collect_scenarios(
    junit_path: Path, source_root: Path, source_map: dict[str, Path]
) -> list[Scenario]:
    tree = ET.parse(junit_path)
    root = tree.getroot()
    scenarios: list[Scenario] = []

    for testcase in root.iter():
        if strip_tag(testcase.tag) != "testcase":
            continue

        classname = testcase.attrib.get("classname", "").strip()
        binary_name = classname.split("::")[-1] if classname else ""
        test_name = testcase.attrib.get("name", "").strip()
        system_out = None

        for child in testcase:
            if strip_tag(child.tag) == "system-out":
                system_out = child.text or ""
                break

        behavior = extract_behavior(system_out)
        derived_from_name = behavior is None
        scenario = behavior or snake_to_sentence(test_name.split("::")[-1])
        original_id = f"{classname}::{test_name}" if classname else test_name

        scenarios.append(
            Scenario(
                domain=derive_domain(binary_name, test_name),
                family=titleize(binary_name) if binary_name else "Tests",
                binary_name=binary_name,
                test_name=test_name,
                original_id=original_id,
                scenario=scenario,
                status=parse_status(testcase),
                derived_from_name=derived_from_name,
                source_file=source_map.get(binary_name),
                source_function=derive_source_function(test_name),
                source_start_line=None,
                source_end_line=None,
            )
        )

    resolve_source_locations(scenarios, source_root)
    return scenarios


def build_markdown_report(
    scenarios: list[Scenario], report_title: str, report_lede: str
) -> str:
    generated_at = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()
    domains: dict[str, list[Scenario]] = defaultdict(list)
    families: dict[str, list[Scenario]] = defaultdict(list)
    for scenario in scenarios:
        domains[scenario.domain].append(scenario)
        families[scenario.family].append(scenario)
    status_counts = compute_status_counts(scenarios)
    derived = sum(1 for scenario in scenarios if scenario.derived_from_name)
    authored = len(scenarios) - derived

    lines: list[str] = []
    lines.append(f"# {report_title}")
    lines.append("")
    lines.append(report_lede)
    lines.append("")
    lines.append(f"Generated at `{generated_at}`.")
    lines.append("")
    lines.append("## Overview")
    lines.append("")
    lines.append(f"- **Total coverage**: {len(scenarios)} scenarios across {len(families)} suites.")
    lines.append(f"- **Status**: {summarize_status_counts(status_counts)}.")
    lines.append(
        f"- **Scenario summaries**: {authored} authored behavior summaries, {derived} derived from test names."
    )

    failures = [scenario for scenario in scenarios if scenario.status == "failed"]
    lines.append("- **Sections**:")
    if failures:
        lines.append(f"  - [Failures](#{slugify_heading('Failures')})")
    for domain in sorted(domains):
        lines.append(f"  - [{domain}](#{slugify_heading(domain)})")

    if failures:
        lines.append("")
        lines.append("## Failures")
        lines.append("")
        for scenario in sorted(failures, key=lambda item: (item.domain, item.scenario.lower())):
            lines.append(f"- {scenario.domain}: {scenario.scenario} (`{scenario.original_id}`)")

    for domain in sorted(domains):
        entries = sorted(
            domains[domain],
            key=lambda item: (STATUS_ORDER[item.status], item.scenario.lower()),
        )
        derived = sum(1 for entry in entries if entry.derived_from_name)
        all_passed = all(entry.status == "passed" for entry in entries)

        lines.append("")
        lines.append(f"## {domain}")
        lines.append("")
        lines.append(summarize_counts(entries))
        if derived:
            lines.append(
                f"Scenario summaries in this section: {len(entries) - derived} authored, {derived} derived from test names."
            )
        lines.append("")

        for scenario in entries:
            sentence = scenario.scenario.rstrip(".") + "."
            line = f"- {sentence}" if all_passed else f"- {scenario.status.capitalize()}: {sentence}"
            source_reference = format_source_reference(scenario)
            if source_reference:
                line += f" Source: `{source_reference}`."
            elif scenario.derived_from_name or scenario.status != "passed":
                line += f" Source: `{scenario.original_id}`."
            lines.append(line)

    lines.append("")
    return "\n".join(lines)


def build_html_report(
    scenarios: list[Scenario], report_title: str, report_lede: str
) -> str:
    generated_at = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()
    domains: dict[str, list[Scenario]] = defaultdict(list)
    families: dict[str, list[Scenario]] = defaultdict(list)
    for scenario in scenarios:
        domains[scenario.domain].append(scenario)
        families[scenario.family].append(scenario)
    status_counts = compute_status_counts(scenarios)
    derived = sum(1 for scenario in scenarios if scenario.derived_from_name)
    authored = len(scenarios) - derived

    overview_items: list[str] = []
    overview_items.append(
        f"<li><strong>Total coverage</strong>: {len(scenarios)} scenarios across {len(families)} suites.</li>"
    )
    overview_items.append(
        f"<li><strong>Status</strong>: {html.escape(summarize_status_counts(status_counts))}.</li>"
    )
    overview_items.append(
        "<li><strong>Scenario summaries</strong>: "
        f"{authored} authored behavior summaries, {derived} derived from test names."
        "</li>"
    )

    section_html: list[str] = []
    for domain in sorted(domains):
        entries = sorted(
            domains[domain],
            key=lambda item: (STATUS_ORDER[item.status], item.scenario.lower()),
        )
        derived = sum(1 for entry in entries if entry.derived_from_name)
        all_passed = all(entry.status == "passed" for entry in entries)
        bullets: list[str] = []

        for scenario in entries:
            sentence = html.escape(scenario.scenario.rstrip(".") + ".")
            meta: list[str] = []
            source_reference = format_source_reference(scenario)
            if source_reference:
                meta.append(f"<code>{html.escape(source_reference)}</code>")
            elif scenario.derived_from_name or scenario.status != "passed":
                meta.append(f"<code>{html.escape(scenario.original_id)}</code>")

            if scenario.derived_from_name:
                meta.append("derived from test name")
            if scenario.status != "passed":
                meta.append(html.escape(scenario.status))

            suffix = ""
            if meta:
                suffix = ' <span class="meta">(' + " · ".join(meta) + ")</span>"

            if all_passed:
                bullets.append(f"<li>{sentence}{suffix}</li>")
            else:
                bullets.append(
                    f'<li><span class="status status-{html.escape(scenario.status)}">{html.escape(scenario.status.capitalize())}</span> {sentence}{suffix}</li>'
                )

        notes = ""
        section_notes: list[str] = []
        if derived:
            section_notes.append(
                f"Scenario summaries in this section: {len(entries) - derived} authored, {derived} derived from test names."
            )
        if section_notes:
            notes = "".join(
                f'<p class="section-note">{html.escape(note)}</p>' for note in section_notes
            )

        section_html.append(
            f"""
            <section class="domain-section" id="{html.escape(slugify_heading(domain))}">
              <h2>{html.escape(domain)}</h2>
              <p class="section-summary">{html.escape(summarize_counts(entries))}</p>
              {notes}
              <ul class="scenario-list">
                {''.join(bullets)}
              </ul>
            </section>
            """
        )

    failures_html = ""
    failures = [scenario for scenario in scenarios if scenario.status == "failed"]
    toc_items: list[str] = []
    if failures:
        toc_items.append('<li><a href="#failures">Failures</a></li>')
    for domain in sorted(domains):
        toc_items.append(
            f'<li><a href="#{html.escape(slugify_heading(domain))}">{html.escape(domain)}</a></li>'
        )
    overview_items.append(
        '<li><strong>Sections</strong><ul class="overview-toc">'
        + "".join(toc_items)
        + "</ul></li>"
    )
    if failures:
        items = []
        for scenario in sorted(failures, key=lambda item: (item.domain, item.scenario.lower())):
            items.append(
                "<li>"
                f"<strong>{html.escape(scenario.domain)}</strong>: "
                f"{html.escape(scenario.scenario)}"
                "</li>"
            )
        failures_html = (
            '<section class="failure-section" id="failures"><h2>Failures</h2><ul class="scenario-list">'
            + "".join(items)
            + "</ul></section>"
        )

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{html.escape(report_title)}</title>
  <link href="https://fonts.googleapis.com/css2?family=Libre+Franklin:wght@300;400;500;600;700&amp;family=JetBrains+Mono:wght@400;500;700&amp;family=Newsreader:ital,opsz,wght@0,6..72,200..800;1,6..72,200..800&amp;display=swap" rel="stylesheet">
  <style>
    :root {{
      --bg: oklch(0.985 0.012 86);
      --card: oklch(0.995 0.006 86);
      --text: oklch(0.19 0.025 255);
      --muted: oklch(0.47 0.025 255);
      --border: oklch(0.88 0.024 238);
      --accent: oklch(0.47 0.12 238);
      --accent-soft: oklch(0.94 0.025 238);
      --fail: oklch(0.54 0.15 28);
      --skip: oklch(0.66 0.13 74);
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Libre Franklin", sans-serif;
      background: linear-gradient(180deg, var(--bg) 0%, oklch(0.955 0.018 86) 100%);
      color: var(--text);
      line-height: 1.6;
    }}
    main {{
      max-width: 980px;
      margin: 0 auto;
      padding: 48px 24px 80px;
    }}
    header {{
      margin-bottom: 32px;
    }}
    h1 {{
      font-size: 2.4rem;
      font-family: "Newsreader", serif;
      line-height: 1.1;
      margin: 0 0 12px;
    }}
    h2 {{
      font-size: 1.35rem;
      font-family: "Newsreader", serif;
      margin: 0 0 12px;
    }}
    p, li {{
      color: var(--muted);
    }}
    .lede {{
      max-width: 70ch;
      margin: 0 0 8px;
    }}
    .timestamp {{
      font-size: 0.9rem;
      color: var(--muted);
      font-family: "JetBrains Mono", monospace;
    }}
    .overview, .domain-section, .failure-section {{
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 18px;
      padding: 24px;
      margin-bottom: 20px;
      box-shadow: 0 2px 12px oklch(0.21 0.025 255 / 0.06);
    }}
    .overview ul, .scenario-list {{
      margin: 0;
      padding-left: 1.2rem;
    }}
    .overview-toc {{
      margin-top: 0.6rem;
      padding-left: 1.2rem;
    }}
    .overview-toc li + li {{
      margin-top: 0.35rem;
    }}
    .overview-toc a {{
      color: var(--accent);
      text-decoration: none;
    }}
    .overview-toc a:hover {{
      text-decoration: underline;
    }}
    .scenario-list li + li {{
      margin-top: 0.5rem;
    }}
    .section-summary, .section-note {{
      margin: 0 0 14px;
      color: var(--muted);
    }}
    .meta {{
      color: var(--muted);
      font-size: 0.92rem;
    }}
    .status {{
      display: inline-block;
      padding: 0.1rem 0.45rem;
      border-radius: 999px;
      font-size: 0.82rem;
      font-weight: 600;
      margin-right: 0.35rem;
    }}
    .status-failed {{
      background: #fee2e2;
      color: var(--fail);
    }}
    .status-skipped {{
      background: #fef3c7;
      color: var(--skip);
    }}
    nav {{
      margin-top: 18px;
      font-size: 0.95rem;
    }}
    nav a {{
      color: var(--accent);
      text-decoration: none;
      margin-right: 16px;
    }}
    nav a:hover {{
      text-decoration: underline;
    }}
  </style>
</head>
<body>
  <main>
    <header>
      <h1>{html.escape(report_title)}</h1>
      <p class="lede">
        {html.escape(report_lede)}
      </p>
      <p class="timestamp">Generated at {html.escape(generated_at)}.</p>
      <nav>
        <a href="index.html">Docs home</a>
        <a href="reports.html">Reports</a>
        <a href="compat.html">Compatibility dashboard</a>
      </nav>
    </header>

    <section class="overview">
      <h2>Overview</h2>
      <ul>
        {''.join(overview_items)}
      </ul>
    </section>

    {failures_html}
    {''.join(section_html)}
  </main>
</body>
</html>
"""


def main() -> int:
    args = parse_args()
    junit_path = Path(args.junit)
    output_path = Path(args.output)
    html_output_path = Path(args.output_html) if args.output_html else None
    source_root = Path(args.source_root).resolve()
    report_title = args.report_title
    report_lede = args.report_lede

    if not junit_path.exists():
        print(f"missing JUnit XML: {junit_path}", file=sys.stderr)
        return 1

    try:
        source_map = parse_source_maps(args.source_map)
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1

    scenarios = collect_scenarios(junit_path, source_root, source_map)
    if not scenarios:
        print(f"no testcases found in JUnit XML: {junit_path}", file=sys.stderr)
        return 1

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        build_markdown_report(scenarios, report_title, report_lede), encoding="utf-8"
    )
    print(f"wrote {output_path}")

    if html_output_path:
        html_output_path.parent.mkdir(parents=True, exist_ok=True)
        html_output_path.write_text(
            build_html_report(scenarios, report_title, report_lede), encoding="utf-8"
        )
        print(f"wrote {html_output_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
