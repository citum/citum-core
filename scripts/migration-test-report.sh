#!/usr/bin/env bash
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JUNIT_PATH="${ROOT_DIR}/target/nextest/report/junit.xml"
REPORT_PATH="${ROOT_DIR}/target/migration-behavior-report.md"
REPORT_HTML_PATH="${ROOT_DIR}/target/migration-behavior-report.html"

if [[ $# -eq 0 ]]; then
  set -- --test date_inference --test substitute_extraction
fi

mkdir -p "$(dirname "${JUNIT_PATH}")" "$(dirname "${REPORT_PATH}")"
rm -f "${JUNIT_PATH}" "${REPORT_PATH}" "${REPORT_HTML_PATH}"

set +e
cargo nextest run --profile report -p citum-migrate "$@"
test_status=$?
set -e

report_status=0
python3 "${ROOT_DIR}/scripts/generate-test-report.py" \
  --junit "${JUNIT_PATH}" \
  --output "${REPORT_PATH}" \
  --output-html "${REPORT_HTML_PATH}" \
  --source-root "${ROOT_DIR}" \
  --report-title "Migration Behavior Coverage" \
  --report-lede "This page is generated from reviewer-facing citum-migrate suites that exercise user-visible migration behavior." \
  --source-map "date_inference=crates/citum-migrate/tests/date_inference.rs" \
  --source-map "substitute_extraction=crates/citum-migrate/tests/substitute_extraction.rs" \
  || report_status=$?

if [[ ${test_status} -ne 0 ]]; then
  exit "${test_status}"
fi

exit "${report_status}"
