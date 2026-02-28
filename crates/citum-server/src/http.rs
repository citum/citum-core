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

/// Build the HTTP router for JSON-RPC requests.
pub fn app() -> Router {
    Router::new().route("/rpc", post(rpc_handler))
}

/// Start the HTTP server on the given port.
pub async fn run_http(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("Citum server listening on http://{}", addr);

    axum::serve(listener, app()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::rpc_handler;
    use axum::{
        Json,
        body::{Body, to_bytes},
        response::IntoResponse,
    };
    use serde_json::json;

    /// Absolute path to the APA style.
    /// `CARGO_MANIFEST_DIR` is the crate root; workspace root is two levels up.
    fn apa_style_path() -> String {
        format!("{}/../../styles/apa-7th.yaml", env!("CARGO_MANIFEST_DIR"))
    }

    /// Minimal bibliography: one book (Hawking 1988) in native Citum schema format.
    fn hawking_refs() -> serde_json::Value {
        json!({
            "ITEM-2": {
                "id": "ITEM-2",
                "class": "monograph",
                "type": "book",
                "title": "A Brief History of Time",
                "author": [{"family": "Hawking", "given": "Stephen"}],
                "issued": "1988"
            }
        })
    }

    async fn response_body_json(response: axum::response::Response<Body>) -> serde_json::Value {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        serde_json::from_slice(&body).expect("response body should be valid JSON")
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_render_citation_returns_ok() {
        let payload = serde_json::from_value(json!({
            "id": 1,
            "method": "render_citation",
            "params": {
                "style_path": apa_style_path(),
                "refs": hawking_refs(),
                "citation": {
                    "id": "cite-1",
                    "items": [{"id": "ITEM-2"}]
                }
            }
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(Json(payload)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = response_body_json(response).await;
        let result = body["result"].as_str().expect("result should be a string");
        assert!(
            result.contains("Hawking") || result.contains("1988"),
            "citation should reference the work: {result}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_render_bibliography_html_returns_ok() {
        let payload = serde_json::from_value(json!({
            "id": 4,
            "method": "render_bibliography",
            "params": {
                "style_path": apa_style_path(),
                "refs": hawking_refs(),
                "output_format": "html"
            }
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(Json(payload)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = response_body_json(response).await;
        assert_eq!(body["result"]["format"], "html");
        let content = body["result"]["content"]
            .as_str()
            .expect("content should be a string");
        assert!(
            content.contains("csln-bibliography"),
            "html bibliography should include wrapper markup"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_unknown_method_returns_bad_request() {
        let payload = serde_json::from_value(json!({
            "id": 2,
            "method": "frobnicate",
            "params": {}
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(Json(payload)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

        let body = response_body_json(response).await;
        assert_eq!(body["id"], 2);
        assert!(
            body["error"]
                .as_str()
                .expect("error should be a string")
                .contains("unknown method")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_missing_field_returns_bad_request() {
        let payload = serde_json::from_value(json!({
            "id": 3,
            "method": "render_bibliography",
            "params": {}
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(Json(payload)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

        let body = response_body_json(response).await;
        assert_eq!(body["id"], 3);
        assert!(
            body["error"]
                .as_str()
                .expect("error should be a string")
                .contains("style_path")
        );
    }
}
