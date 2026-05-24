/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "bin")]

#[cfg(feature = "http")]
use citum_server::http;
use citum_server::rpc;
use clap::Parser;
use clap::builder::styling::{AnsiColor, Effects, Styles};

const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[cfg(feature = "http")]
const ABOUT: &str = "JSON-RPC and HTTP server for citation, bibliography, and document processing";

#[cfg(not(feature = "http"))]
const ABOUT: &str = "JSON-RPC server for citation, bibliography, and document processing";

#[cfg(feature = "http")]
const LONG_ABOUT: &str = "citum-server provides a programmable interface for citation
processing.

It supports both a persistent JSON-RPC mode via stdin/stdout
and HTTP server mode for remote processing.

EXAMPLES:
  Run in default JSON-RPC mode (stdin/stdout):
    citum-server

  Run as an HTTP server on a specific port:
    citum-server --http --port 8081";

#[cfg(not(feature = "http"))]
const LONG_ABOUT: &str = "citum-server provides a programmable interface for citation
processing.

This build supports persistent JSON-RPC mode via stdin/stdout.

EXAMPLES:
  Run in JSON-RPC mode (stdin/stdout):
    citum-server";

#[derive(Parser)]
#[command(
    name = "citum-server",
    author,
    version,
    about = ABOUT,
    long_about = LONG_ABOUT,
    styles = CLAP_STYLES,
)]
struct Cli {
    /// Enable HTTP server mode (default build includes the 'http' feature)
    #[cfg(feature = "http")]
    #[arg(long)]
    http: bool,

    /// HTTP port to listen on
    #[cfg(feature = "http")]
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "http"))]
    let _cli = Cli::parse();

    #[cfg(feature = "http")]
    let cli = Cli::parse();

    #[cfg(feature = "http")]
    if cli.http {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        return runtime.block_on(http::run_http(cli.port));
    }

    rpc::run_stdio().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
