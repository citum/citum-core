/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use crate::rpc::{RpcRequest, dispatch};
use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::json;
use std::net::SocketAddr;

/// HTTP handler for JSON-RPC requests.
/// Dispatches to the same RPC logic as stdin/stdout.
async fn rpc_handler(Json(payload): Json<RpcRequest>) -> impl IntoResponse {
    match dispatch(payload.clone()) {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err((id, error)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "id": id,
                "error": error
            })),
        ),
    }
}

/// Start the HTTP server on the given port.
pub async fn run_http(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/rpc", post(rpc_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("Citum server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
