//! Names and concept ids attached to every response.

use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;

pub const HANDLER_HEADER: HeaderName = HeaderName::from_static("x-rust-handler");
pub const CONCEPTS_HEADER: HeaderName = HeaderName::from_static("x-rust-concepts");

#[derive(Clone, Copy)]
pub struct HandlerMeta {
    pub name: &'static str,
    pub concepts: &'static str,
}

pub fn meta_for(method: &str, path: &str) -> HandlerMeta {
    if method == "GET" && path == "/api/health" {
        return HandlerMeta {
            name: "health",
            concepts: "async-await,tokio",
        };
    }
    if method == "GET" && path == "/api/expenses" {
        return HandlerMeta {
            name: "list_expenses",
            concepts: "borrowing,iterators,closures,serde",
        };
    }
    if method == "POST" && path == "/api/expenses" {
        return HandlerMeta {
            name: "create_expense",
            concepts: "ownership,mut-borrow,result,question-mark,newtype,integer-money,structs",
        };
    }
    if method == "GET" && is_expense_id(path) {
        return HandlerMeta {
            name: "get_expense",
            concepts: "option,borrowing",
        };
    }
    if method == "DELETE" && is_expense_id(path) {
        return HandlerMeta {
            name: "delete_expense",
            concepts: "move,ownership,mut-borrow",
        };
    }
    if method == "GET" && path == "/api/summary" {
        return HandlerMeta {
            name: "summary",
            concepts: "iterators,btreemap,hashmap",
        };
    }
    if method == "PUT" && is_budget_path(path) {
        return HandlerMeta {
            name: "set_budget",
            concepts: "hashmap,newtype,integer-money",
        };
    }
    HandlerMeta {
        name: "not_found",
        concepts: "option",
    }
}

fn is_expense_id(path: &str) -> bool {
    match path.strip_prefix("/api/expenses/") {
        Some(rest) => !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()),
        None => false,
    }
}

fn is_budget_path(path: &str) -> bool {
    path.strip_prefix("/api/budgets/")
        .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
}

pub async fn attach_headers(request: Request<Body>, next: Next) -> Response {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    let meta = meta_for(&method, &path);
    if let Ok(name) = HeaderValue::from_str(meta.name) {
        response.headers_mut().insert(HANDLER_HEADER, name);
    }
    if let Ok(concepts) = HeaderValue::from_str(meta.concepts) {
        response.headers_mut().insert(CONCEPTS_HEADER, concepts);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_map_to_handlers() {
        assert_eq!(meta_for("POST", "/api/expenses").name, "create_expense");
        assert_eq!(meta_for("DELETE", "/api/expenses/3").name, "delete_expense");
        assert_eq!(meta_for("PUT", "/api/budgets/food").name, "set_budget");
        assert_eq!(meta_for("GET", "/missing").name, "not_found");
        assert_eq!(meta_for("GET", "/api/expenses/nope").name, "not_found");
    }
}
