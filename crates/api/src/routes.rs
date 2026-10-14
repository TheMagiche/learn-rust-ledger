//! Router, CORS for the Vite dev server, and the embedded UI.

use axum::body::Body;
use axum::http::{header, HeaderValue, Method, StatusCode, Uri};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers::{
    create_expense, delete_expense, get_expense, health, list_expenses, set_budget, summary,
};
use crate::learn::{attach_headers, CONCEPTS_HEADER, HANDLER_HEADER};
use crate::state::SharedState;

pub fn router(state: SharedState, dev: bool) -> Router {
    let mut app = Router::new()
        .route("/api/health", get(health))
        .route("/api/expenses", get(list_expenses).post(create_expense))
        .route(
            "/api/expenses/{id}",
            get(get_expense).delete(delete_expense),
        )
        .route("/api/summary", get(summary))
        .route("/api/budgets/{category}", put(set_budget));

    if dev {
        app = app.fallback(dev_fallback);
    } else {
        app = app.fallback(embedded_fallback);
    }

    app = app
        .layer(middleware::from_fn(attach_headers))
        .layer(TraceLayer::new_for_http());
    if dev {
        app = app.layer(dev_cors());
    }
    app.with_state(state)
}

fn dev_cors() -> CorsLayer {
    // `--dev` lets the Vite app on port 5173 read the teaching headers.
    CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .expose_headers([HANDLER_HEADER, CONCEPTS_HEADER])
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

async fn dev_fallback(uri: Uri) -> Response {
    api_or_message(
        &uri,
        StatusCode::NOT_FOUND,
        "spend API is in dev mode; open the Vite UI",
    )
}

async fn embedded_fallback(uri: Uri) -> Response {
    if uri.path().starts_with("/api/") || uri.path() == "/api" {
        return missing_api();
    }
    match asset_for(uri.path()) {
        Some(response) => response,
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

fn api_or_message(uri: &Uri, status: StatusCode, message: &'static str) -> Response {
    if uri.path().starts_with("/api/") || uri.path() == "/api" {
        missing_api()
    } else {
        (status, message).into_response()
    }
}

fn missing_api() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody {
            error: "not found".to_string(),
        }),
    )
        .into_response()
}

#[derive(rust_embed::Embed)]
#[folder = "../../web/dist"]
struct Assets;

fn asset_for(path: &str) -> Option<Response> {
    let relative = path.trim_start_matches('/');
    if relative.contains("..") {
        return None;
    }
    if relative.is_empty() {
        return file_response("index.html");
    }
    if let Some(response) = file_response(relative) {
        return Some(response);
    }
    if !relative.contains('.') {
        return file_response("index.html");
    }
    None
}

fn file_response(path: &str) -> Option<Response> {
    let file = Assets::get(path)?;
    let mime = mime_for(path);
    let response = (
        [(header::CONTENT_TYPE, HeaderValue::from_static(mime))],
        Body::from(file.data.into_owned()),
    );
    Some(response.into_response())
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("json") => "application/json",
        Some("woff2") => "font/woff2",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
