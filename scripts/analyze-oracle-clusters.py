#!/usr/bin/env python3
"""Cluster oracle bibliography failures by first-divergence pattern.

Part of the mid-tuning audit loop for embedded styles: before spending
effort on entry-by-entry fixes, run the oracle with --json, feed the
report here, and triage the resulting clusters (style fix / conversion
fix / engine fix / registered divergence) biggest-first.

Usage:
  node scripts/oracle.js <style.csl> --json \
      --refs-fixture <refs.json> --citations-fixture <cites.json> \
      > /tmp/oracle.json
  python3 scripts/analyze-oracle-clusters.py /tmp/oracle.json
"""

import json
import re
import sys


def strip_label(text):
    """Drop the leading [n] citation-number label so diffs start at content."""
    return re.sub(r"^\[[^\]]*\]", "", text)


def first_divergence(oracle, citum):
    i = 0
    while i < min(len(oracle), len(citum)) and oracle[i] == citum[i]:
        i += 1
    return i


def classify(oracle, citum):
    """Return a coarse cluster key for one failed entry."""
    o = strip_label(oracle)
    c = strip_label(citum)
    tail = o[len(c):] if o.startswith(c) else None
    if tail and re.fullmatch(r"[：:][0-9，,\-–—]+", tail):
        return "trailing-cited-pages-missing"
    i = first_divergence(o, c)
    ctx_o = o[max(0, i - 12):i + 24]
    ctx_c = c[max(0, i - 12):i + 24]
    if re.search(r"\d{4}, ", ctx_c) or re.search(r"[（(\[]\d{4}-\d{2}", ctx_o):
        return "date-format"
    if "et al" in ctx_o and "等" in ctx_c or "等" in ctx_o and "et al" in ctx_c:
        return "et-al-term-language"
    if re.search(r"\. \[[A-Z]{1,2}[\]/]", ctx_c) and not re.search(r"\. \[[A-Z]{1,2}[\]/]", ctx_o):
        return "carrier-marker-spurious-delimiter"
    return f"other @{i}: oracle<<{ctx_o}>> citum<<{ctx_c}>>"


def main():
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    with open(sys.argv[1], encoding="utf-8") as handle:
        report = json.load(handle)
    entries = (report.get("bibliography") or {}).get("entries") or []
    failures = [e for e in entries if not e.get("match")]
    clusters = {}
    for entry in failures:
        key = classify(entry.get("oracle") or "", entry.get("citum") or "")
        clusters.setdefault(key, []).append(entry.get("id") or entry.get("index"))
    total = (report.get("bibliography") or {}).get("total")
    print(f"failures: {len(failures)} / {total}")
    for key in sorted(clusters, key=lambda k: -len(clusters[k])):
        ids = clusters[key]
        sample = ", ".join(str(x) for x in ids[:6])
        print(f"{len(ids):4}  {key}  [{sample}{', …' if len(ids) > 6 else ''}]")
    return 0


if __name__ == "__main__":
    sys.exit(main())
