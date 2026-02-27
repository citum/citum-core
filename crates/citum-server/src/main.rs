/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

mod error;
mod rpc;

#[cfg(feature = "http")]
mod http;

use std::env;

/// Parse command-line arguments for server mode.
/// Returns (use_http, port) tuple.
fn parse_args() -> (bool, u16) {
    let args: Vec<String> = env::args().collect();
    let mut use_http = false;
    let mut port = 8080u16;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--http" => use_http = true,
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                    }
                    i += 1;
                }
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("citum-server {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    (use_http, port)
}

fn print_help() {
    eprintln!(
        "citum-server {}

USAGE:
    citum-server [OPTIONS]

OPTIONS:
    --http              Enable HTTP server mode (requires 'http' feature)
    --port <PORT>       HTTP port to listen on (default: 8080)
    --help              Print help information
    --version           Print version information

Without --http, the server runs in stdin/stdout JSON-RPC mode.
",
        env!("CARGO_PKG_VERSION")
    );
}

#[cfg(feature = "async")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (use_http, port) = parse_args();

    if use_http {
        #[cfg(feature = "http")]
        {
            http::run_http(port).await
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
    let (use_http, _port) = parse_args();

    if use_http {
        eprintln!("Error: --http requires the 'http' feature to be enabled");
        eprintln!("Build with: cargo build --features http");
        std::process::exit(1);
    }

    rpc::run_stdio().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
