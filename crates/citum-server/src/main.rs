#![allow(missing_docs, reason = "bin")]

/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

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

#[derive(Parser)]
#[command(
    name = "citum-server",
    author,
    version,
    about = "JSON-RPC and HTTP server for citation, bibliography, and document processing",
    long_about = "citum-server provides a programmable interface for citation\n\
                  processing.\n\n\
                  It supports both a persistent JSON-RPC mode via stdin/stdout\n\
                  and an optional HTTP server mode for remote processing.\n\n\
                  EXAMPLES:\n  \
                  Run in default JSON-RPC mode (stdin/stdout):\n    \
                  citum-server\n\n  \
                  Run as an HTTP server on a specific port:\n    \
                  citum-server --http --port 8081",
    styles = CLAP_STYLES,
)]
struct Cli {
    /// Enable HTTP server mode (requires 'http' feature)
    #[arg(long)]
    http: bool,

    /// HTTP port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[cfg(feature = "async")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.http {
        #[cfg(feature = "http")]
        {
            http::run_http(cli.port).await
        }
        #[cfg(not(feature = "http"))]
        {
            eprintln!("Error: --http requires the 'http' feature to be enabled");
            eprintln!("Build with: cargo build --features http");
            std::process::exit(1);
        }
    } else {
        rpc::run_stdio().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.http {
        eprintln!("Error: --http requires the 'http' feature to be enabled");
        eprintln!("Build with: cargo build --features http");
        std::process::exit(1);
    }

    rpc::run_stdio().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
