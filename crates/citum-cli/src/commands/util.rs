/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Shared CLI helpers: interactive prompts and input validation.

use super::CliResult;
use std::io::{self, IsTerminal, Write};

/// Prompt the user for confirmation. Returns `Ok(true)` on `y`/`yes`.
///
/// Errors when stdin is not a TTY — non-interactive callers must pass `--yes`.
pub(super) fn confirm(prompt: &str) -> CliResult<bool> {
    if !io::stdin().is_terminal() {
        return Err(format!("{prompt} Use --yes to run non-interactively.").into());
    }
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    Ok(matches!(
        response.trim().to_lowercase().as_str(),
        "y" | "yes"
    ))
}

/// Reject names containing path separators, `..`, or non-alphanumeric chars.
pub(super) fn validate_resource_name(name: &str) -> CliResult {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
    {
        Ok(())
    } else {
        Err(
            format!("Invalid name: '{name}'. Names must be alphanumeric, hyphens, or underscores.")
                .into(),
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn test_validate_resource_name() {
        assert!(validate_resource_name("apa").is_ok());
        assert!(validate_resource_name("apa-7th").is_ok());
        assert!(validate_resource_name("chicago_fullnote").is_ok());
        assert!(validate_resource_name("").is_err());
        assert!(validate_resource_name("..").is_err());
        assert!(validate_resource_name("../../etc/passwd").is_err());
        assert!(validate_resource_name("styles/apa").is_err());
        assert!(validate_resource_name("apa.yaml").is_err());
        assert!(validate_resource_name("my registry!").is_err());
    }
}
