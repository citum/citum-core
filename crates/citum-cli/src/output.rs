use citum_schema::lint::{LintReport, LintSeverity};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub(crate) fn print_lint_report(label: &str, report: &LintReport) {
    if report.findings.is_empty() {
        println!("{label}: ok");
        return;
    }

    println!("{label}:");
    for finding in &report.findings {
        let level = match finding.severity {
            LintSeverity::Warning => "warning",
            LintSeverity::Error => "error",
        };
        println!("  {level}: {}: {}", finding.path, finding.message);
    }
}

pub(crate) fn write_output(output: &str, path: Option<&PathBuf>) -> Result<(), Box<dyn Error>> {
    if let Some(file) = path {
        fs::write(file, output)?;
    } else {
        println!("{output}");
    }
    Ok(())
}
