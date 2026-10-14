//! Every route, including the failures a beginner is likely to trigger.

use std::sync::Arc;

use chrono::Datelike;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use spend_api::{router, AppState};
use spend_store::{MemoryStore, Store};
use tokio::sync::Mutex;
use tower::ServiceExt;

fn app(dev: bool) -> axum::Router {
    let store = MemoryStore::default();
    let ledger = store.load().expect("load");
    let state = Arc::new(AppState {
        ledger: Mutex::new(ledger),
        store: Box::new(store),
    });
    router(state, dev)
}

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<&str>,
) -> (StatusCode, String, Option<String>, Option<String>) {
    let mut builder = Request::builder().method(method).uri(uri);
    let request_body = match body {
        Some(text) => {
            builder = builder.header("content-type", "application/json");
            Body::from(text.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(request_body).expect("request"))
        .await
        .expect("response");
    let status = response.status();
    let handler = response
        .headers()
        .get("x-rust-handler")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let concepts = response
        .headers()
        .get("x-rust-concepts")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let text = String::from_utf8(bytes.to_vec()).expect("utf8");
    (status, text, handler, concepts)
}

fn json_body(text: &str) -> Value {
    if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(text).unwrap_or(Value::String(text.to_string()))
    }
}

#[tokio::test]
async fn health_and_teaching_headers() {
    let app = app(true);
    let (status, text, handler, concepts) = call(&app, "GET", "/api/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json_body(&text), json!({"ok": true}));
    assert_eq!(handler.as_deref(), Some("health"));
    assert_eq!(concepts.as_deref(), Some("async-await,tokio"));
}

#[tokio::test]
async fn create_list_get_and_delete() {
    let app = app(false);
    let (status, text, handler, _) = call(
        &app,
        "POST",
        "/api/expenses",
        Some(r#"{"amount":"12.50","category":"food","note":"lunch","date":"2026-10-14"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(handler.as_deref(), Some("create_expense"));
    let created = json_body(&text);
    assert_eq!(created["amount"], 1250);
    assert_eq!(created["display"], "12.50");
    assert_eq!(created["category"], "food");
    assert_eq!(created["id"], 1);

    let (status, _, _, _) = call(
        &app,
        "POST",
        "/api/expenses",
        Some(r#"{"amount":"4.00","category":"Transport","note":"bus","date":"2026-09-01"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, text, handler, _) = call(
        &app,
        "GET",
        "/api/expenses?month=2026-10&category=food&search=lunch",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(handler.as_deref(), Some("list_expenses"));
    let rows = json_body(&text);
    assert_eq!(rows.as_array().expect("array").len(), 1);
    assert_eq!(rows[0]["note"], "lunch");

    let (status, text, handler, _) = call(&app, "GET", "/api/expenses/1", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(handler.as_deref(), Some("get_expense"));
    assert_eq!(json_body(&text)["display"], "12.50");

    let (status, text, handler, _) = call(&app, "DELETE", "/api/expenses/1", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(text.is_empty());
    assert_eq!(handler.as_deref(), Some("delete_expense"));

    let (status, text, _, _) = call(&app, "GET", "/api/expenses/1", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json_body(&text)["error"], "expense 1 not found");
}

#[tokio::test]
async fn rejects_bad_input() {
    let app = app(false);
    let cases = [
        (
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"0","category":"food"}"#),
            StatusCode::BAD_REQUEST,
            "amount must be greater than zero",
        ),
        (
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"abc","category":"food"}"#),
            StatusCode::BAD_REQUEST,
            "invalid amount: abc",
        ),
        (
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"1.234","category":"food"}"#),
            StatusCode::BAD_REQUEST,
            "invalid amount: 1.234",
        ),
        (
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"-1","category":"food"}"#),
            StatusCode::BAD_REQUEST,
            "invalid amount: -1",
        ),
        (
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"10","category":"spaceship"}"#),
            StatusCode::BAD_REQUEST,
            "unknown category: spaceship",
        ),
        (
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"10","category":"food","date":"yesterday"}"#),
            StatusCode::BAD_REQUEST,
            "invalid date: yesterday",
        ),
        (
            "POST",
            "/api/expenses",
            Some("not-json"),
            StatusCode::BAD_REQUEST,
            "invalid JSON body:",
        ),
        (
            "GET",
            "/api/expenses?month=2026-13",
            None,
            StatusCode::BAD_REQUEST,
            "invalid month: 2026-13",
        ),
        (
            "GET",
            "/api/expenses/nope",
            None,
            StatusCode::BAD_REQUEST,
            "invalid expense id",
        ),
        (
            "DELETE",
            "/api/expenses/99",
            None,
            StatusCode::NOT_FOUND,
            "expense 99 not found",
        ),
        (
            "PUT",
            "/api/budgets/food",
            Some(r#"{"amount":"nope"}"#),
            StatusCode::BAD_REQUEST,
            "invalid amount: nope",
        ),
        (
            "PUT",
            "/api/budgets/nope",
            Some(r#"{"amount":"10"}"#),
            StatusCode::BAD_REQUEST,
            "unknown category: nope",
        ),
    ];

    for (method, uri, body, expected_status, expected_message) in cases {
        let (status, text, _, _) = call(&app, method, uri, body).await;
        assert_eq!(status, expected_status, "{method} {uri} -> {text}");
        let message = json_body(&text)["error"].as_str().unwrap_or("").to_string();
        assert!(
            message.contains(expected_message),
            "{method} {uri} message {message} should contain {expected_message}"
        );
    }
}

#[tokio::test]
async fn summary_and_budget() {
    let app = app(false);
    let today = chrono::Local::now().date_naive();
    let month = format!("{:04}-{:02}", today.year(), today.month());
    let create = format!(r#"{{"amount":"80.00","category":"food","date":"{today}"}}"#);
    let (status, _, _, _) = call(&app, "POST", "/api/expenses", Some(&create)).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, text, handler, concepts) = call(
        &app,
        "PUT",
        "/api/budgets/food",
        Some(r#"{"amount":"50.00"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(handler.as_deref(), Some("set_budget"));
    assert!(concepts.unwrap_or_default().contains("integer-money"));
    let budget = json_body(&text);
    assert_eq!(budget["over"], true);
    assert_eq!(budget["budget"]["display"], "50.00");
    assert_eq!(budget["spent"]["amount"], 8000);

    let (status, text, handler, _) =
        call(&app, "GET", &format!("/api/summary?month={month}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(handler.as_deref(), Some("summary"));
    let summary = json_body(&text);
    assert_eq!(summary["total"]["display"], "80.00");
    assert_eq!(summary["byCategory"][0]["category"], "food");
    assert_eq!(summary["budgets"][0]["over"], true);
}

#[tokio::test]
async fn dev_cors_exposes_teaching_headers() {
    let app = app(true);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health")
                .header("origin", "http://localhost:5173")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let expose = response
        .headers()
        .get("access-control-expose-headers")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    assert!(expose.contains("x-rust-handler"), "{expose}");
    assert!(expose.contains("x-rust-concepts"), "{expose}");
}

#[tokio::test]
async fn embedded_ui_and_unknown_api() {
    let embedded = app(false);
    let (status, text, _, _) = call(&embedded, "GET", "/", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        text.to_lowercase().contains("html") || text.contains("spend"),
        "{text}"
    );

    let (status, text, handler, _) = call(&embedded, "GET", "/api/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json_body(&text)["error"], "not found");
    assert_eq!(handler.as_deref(), Some("not_found"));

    let dev_app = app(true);
    let (status, text, _, _) = call(&dev_app, "GET", "/", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(text.contains("dev mode"), "{text}");
}
