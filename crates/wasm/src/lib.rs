//! WebAssembly client engine for the spend ledger.
//!
//! LEARN(wasm): this crate compiles pure Rust into a WebAssembly module that runs
//! directly inside the browser on GitHub Pages without requiring a backend server.

use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use spend_core::{
    month_key, parse_user_date, parse_user_month, BudgetStatus, Category, Expense, Filter, Money,
    NewExpense, SpendError,
};
use wasm_bindgen::prelude::*;

#[allow(dead_code)]
const STORAGE_KEY: &str = "spend.ledger";

thread_local! {
    static ENGINE: RefCell<WasmEngine> = RefCell::new(WasmEngine::new());
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmResponse {
    pub status: u16,
    pub handler: String,
    pub concepts: Vec<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateExpenseBody {
    amount: String,
    category: String,
    note: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AmountBody {
    amount: String,
}

#[derive(Serialize)]
pub struct HealthBody {
    ok: bool,
}

#[derive(Serialize)]
pub struct ExpenseJson {
    id: u32,
    amount: u64,
    display: String,
    category: Category,
    note: Option<String>,
    date: NaiveDate,
}

#[derive(Serialize)]
pub struct MoneyJson {
    amount: u64,
    display: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTotalJson {
    category: Category,
    amount: u64,
    display: String,
}

#[derive(Serialize)]
pub struct BudgetJson {
    category: Category,
    budget: MoneyJson,
    spent: MoneyJson,
    over: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryJson {
    total: MoneyJson,
    by_category: Vec<CategoryTotalJson>,
    budgets: Vec<BudgetJson>,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[wasm_bindgen]
pub struct WasmEngine {
    ledger: spend_core::Ledger,
}

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let ledger = load_ledger().unwrap_or_default();
        Self { ledger }
    }

    #[wasm_bindgen]
    pub fn handle_request(
        &mut self,
        method: &str,
        url: &str,
        body: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let response = self.route(method, url, body.as_deref());
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        response
            .serialize(&serializer)
            .map_err(|err| JsValue::from_str(&format!("serialization error: {err}")))
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.ledger = spend_core::Ledger::default();
        save_ledger(&self.ledger);
        Ok(())
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub fn handle_request(method: &str, url: &str, body: Option<String>) -> Result<JsValue, JsValue> {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        engine.handle_request(method, url, body)
    })
}

#[wasm_bindgen]
pub fn reset_ledger() -> Result<(), JsValue> {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        engine.reset()
    })
}

impl WasmEngine {
    pub fn route(&mut self, method: &str, url: &str, body: Option<&str>) -> WasmResponse {
        let (path, query) = parse_url(url);

        if method == "GET" && path == "/api/health" {
            return WasmResponse {
                status: 200,
                handler: "health".to_string(),
                concepts: vec!["async-await".into(), "tokio".into()],
                data: serde_json::to_value(HealthBody { ok: true }).unwrap(),
            };
        }

        if method == "GET" && path == "/api/expenses" {
            let filter = match filter_from_query(&query) {
                Ok(f) => f,
                Err(err) => {
                    return error_response(
                        "list_expenses",
                        &["borrowing", "iterators", "closures", "serde"],
                        400,
                        &err,
                    )
                }
            };
            let rows: Vec<ExpenseJson> = self
                .ledger
                .list(&filter)
                .into_iter()
                .map(expense_json)
                .collect();
            return WasmResponse {
                status: 200,
                handler: "list_expenses".to_string(),
                concepts: vec![
                    "borrowing".into(),
                    "iterators".into(),
                    "closures".into(),
                    "serde".into(),
                ],
                data: serde_json::to_value(rows).unwrap(),
            };
        }

        if method == "POST" && path == "/api/expenses" {
            let concepts = [
                "ownership",
                "mut-borrow",
                "result",
                "question-mark",
                "newtype",
                "integer-money",
                "structs",
            ];
            let body_str = body.unwrap_or("");
            let parsed_body: CreateExpenseBody = match serde_json::from_str(body_str) {
                Ok(b) => b,
                Err(err) => {
                    return error_response(
                        "create_expense",
                        &concepts,
                        400,
                        &format!("invalid JSON body: {err}"),
                    )
                }
            };
            let amount = match Money::parse(&parsed_body.amount) {
                Ok(a) => a,
                Err(err) => {
                    return error_response("create_expense", &concepts, 400, &err.to_string())
                }
            };
            let category = match Category::from_str(parsed_body.category.trim()) {
                Ok(c) => c,
                Err(err) => {
                    return error_response("create_expense", &concepts, 400, &err.to_string())
                }
            };
            let date = match parsed_body.date {
                Some(value) => match parse_user_date(&value) {
                    Ok(d) => d,
                    Err(err) => {
                        return error_response("create_expense", &concepts, 400, &err.to_string())
                    }
                },
                None => today(),
            };
            let draft = NewExpense {
                amount,
                category,
                note: parsed_body.note,
                date,
            };
            let created = match self.ledger.add(draft) {
                Ok(e) => e.clone(),
                Err(err) => {
                    return error_response("create_expense", &concepts, 400, &err.to_string())
                }
            };
            save_ledger(&self.ledger);
            return WasmResponse {
                status: 201,
                handler: "create_expense".to_string(),
                concepts: concepts.iter().map(|s| s.to_string()).collect(),
                data: serde_json::to_value(expense_json(&created)).unwrap(),
            };
        }

        if let Some(id_str) = path.strip_prefix("/api/expenses/") {
            if !id_str.is_empty() && id_str.bytes().all(|b| b.is_ascii_digit()) {
                let id = id_str.parse::<u32>().unwrap_or(0);
                if method == "GET" {
                    let concepts = ["option", "borrowing"];
                    return match self.ledger.get(id) {
                        Some(e) => WasmResponse {
                            status: 200,
                            handler: "get_expense".to_string(),
                            concepts: concepts.iter().map(|s| s.to_string()).collect(),
                            data: serde_json::to_value(expense_json(e)).unwrap(),
                        },
                        None => error_response(
                            "get_expense",
                            &concepts,
                            404,
                            &SpendError::NotFound(id).to_string(),
                        ),
                    };
                }
                if method == "DELETE" {
                    let concepts = ["move", "ownership", "mut-borrow"];
                    return match self.ledger.remove(id) {
                        Ok(_) => {
                            save_ledger(&self.ledger);
                            WasmResponse {
                                status: 204,
                                handler: "delete_expense".to_string(),
                                concepts: concepts.iter().map(|s| s.to_string()).collect(),
                                data: serde_json::Value::Null,
                            }
                        }
                        Err(err) => {
                            error_response("delete_expense", &concepts, 404, &err.to_string())
                        }
                    };
                }
            } else if method == "GET" || method == "DELETE" {
                let handler = if method == "GET" {
                    "get_expense"
                } else {
                    "delete_expense"
                };
                let concepts = if method == "GET" {
                    &["option", "borrowing"][..]
                } else {
                    &["move", "ownership", "mut-borrow"][..]
                };
                return error_response(handler, concepts, 400, "invalid expense id");
            }
        }

        if method == "GET" && path == "/api/summary" {
            let concepts = ["iterators", "btreemap", "hashmap"];
            let month = match query.get("month") {
                Some(value) if !value.trim().is_empty() => match parse_user_month(value) {
                    Ok(m) => m,
                    Err(err) => return error_response("summary", &concepts, 400, &err.to_string()),
                },
                _ => current_month(),
            };
            let filter = Filter {
                month: Some(month.clone()),
                ..Filter::default()
            };
            let total = self.ledger.total(&filter);
            let by_category = self
                .ledger
                .by_category(Some(&month))
                .into_iter()
                .map(|(category, money)| CategoryTotalJson {
                    category,
                    amount: money.cents(),
                    display: money.to_string(),
                })
                .collect();
            let budgets = self
                .ledger
                .budget_status(&month)
                .iter()
                .map(budget_json)
                .collect();
            return WasmResponse {
                status: 200,
                handler: "summary".to_string(),
                concepts: concepts.iter().map(|s| s.to_string()).collect(),
                data: serde_json::to_value(SummaryJson {
                    total: money_json(total),
                    by_category,
                    budgets,
                })
                .unwrap(),
            };
        }

        if method == "PUT" && path.starts_with("/api/budgets/") {
            let concepts = ["hashmap", "newtype", "integer-money"];
            let category_text = match path.strip_prefix("/api/budgets/") {
                Some(rest) if !rest.is_empty() && !rest.contains('/') => rest,
                _ => {
                    return error_response("set_budget", &concepts, 400, "invalid budget category")
                }
            };
            let category = match Category::from_str(category_text.trim()) {
                Ok(c) => c,
                Err(err) => return error_response("set_budget", &concepts, 400, &err.to_string()),
            };
            let body_str = body.unwrap_or("");
            let parsed_body: AmountBody = match serde_json::from_str(body_str) {
                Ok(b) => b,
                Err(err) => {
                    return error_response(
                        "set_budget",
                        &concepts,
                        400,
                        &format!("invalid JSON body: {err}"),
                    )
                }
            };
            let amount = match Money::parse(&parsed_body.amount) {
                Ok(a) => a,
                Err(err) => return error_response("set_budget", &concepts, 400, &err.to_string()),
            };
            let month = match query.get("month") {
                Some(value) if !value.trim().is_empty() => match parse_user_month(value) {
                    Ok(m) => m,
                    Err(err) => {
                        return error_response("set_budget", &concepts, 400, &err.to_string())
                    }
                },
                _ => current_month(),
            };
            self.ledger.set_budget(category, amount);
            save_ledger(&self.ledger);
            let status = match self
                .ledger
                .budget_status(&month)
                .into_iter()
                .find(|row| row.category == category)
            {
                Some(s) => s,
                None => {
                    return error_response(
                        "set_budget",
                        &concepts,
                        400,
                        &SpendError::UnknownCategory(category.to_string()).to_string(),
                    )
                }
            };
            return WasmResponse {
                status: 200,
                handler: "set_budget".to_string(),
                concepts: concepts.iter().map(|s| s.to_string()).collect(),
                data: serde_json::to_value(budget_json(&status)).unwrap(),
            };
        }

        WasmResponse {
            status: 404,
            handler: "not_found".to_string(),
            concepts: vec!["option".into()],
            data: serde_json::to_value(ErrorBody {
                error: "not found".to_string(),
            })
            .unwrap(),
        }
    }
}

fn error_response(handler: &str, concepts: &[&str], status: u16, message: &str) -> WasmResponse {
    WasmResponse {
        status,
        handler: handler.to_string(),
        concepts: concepts.iter().map(|s| s.to_string()).collect(),
        data: serde_json::to_value(ErrorBody {
            error: message.to_string(),
        })
        .unwrap(),
    }
}

fn expense_json(expense: &Expense) -> ExpenseJson {
    ExpenseJson {
        id: expense.id,
        amount: expense.amount.cents(),
        display: expense.amount.to_string(),
        category: expense.category,
        note: expense.note.clone(),
        date: expense.date,
    }
}

fn money_json(money: Money) -> MoneyJson {
    MoneyJson {
        amount: money.cents(),
        display: money.to_string(),
    }
}

fn budget_json(status: &BudgetStatus) -> BudgetJson {
    BudgetJson {
        category: status.category,
        budget: money_json(status.budget),
        spent: money_json(status.spent),
        over: status.over,
    }
}

fn today() -> NaiveDate {
    #[cfg(target_arch = "wasm32")]
    {
        let now = js_sys::Date::new_0();
        let year = now.get_full_year() as i32;
        let month = (now.get_month() + 1) as u32;
        let day = now.get_date() as u32;
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 10, 14).expect("valid fallback date"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Local::now().date_naive()
    }
}

fn current_month() -> String {
    month_key(today())
}

fn filter_from_query(query: &HashMap<String, String>) -> Result<Filter, String> {
    let category = match query.get("category") {
        Some(value) if !value.trim().is_empty() => {
            Some(Category::from_str(value).map_err(|e| e.to_string())?)
        }
        _ => None,
    };
    let month = match query.get("month") {
        Some(value) if !value.trim().is_empty() => {
            Some(parse_user_month(value).map_err(|e| e.to_string())?)
        }
        _ => None,
    };
    let search = query
        .get("search")
        .cloned()
        .filter(|v| !v.trim().is_empty());
    Ok(Filter {
        category,
        month,
        search,
    })
}

fn parse_url(url: &str) -> (&str, HashMap<String, String>) {
    let (path, query_str) = match url.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (url, None),
    };

    let mut params = HashMap::new();
    if let Some(query) = query_str {
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""),
            };
            let decoded_key = urlencoding_decode(key);
            let decoded_value = urlencoding_decode(value);
            params.insert(decoded_key, decoded_value);
        }
    }

    (path, params)
}

fn urlencoding_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(byte) =
                    u8::from_str_radix(std::str::from_utf8(&[h1, h2]).unwrap_or(""), 16)
                {
                    bytes.push(byte);
                    continue;
                }
                bytes.push(b'%');
                bytes.push(h1);
                bytes.push(h2);
            } else {
                bytes.push(b'%');
                if let Some(h1) = h1 {
                    bytes.push(h1);
                }
            }
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| s.to_string())
}

fn load_ledger() -> Option<spend_core::Ledger> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        let text = storage.get_item(STORAGE_KEY).ok()??;
        if text.trim().is_empty() {
            return None;
        }
        serde_json::from_str(&text).ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn save_ledger(_ledger: &spend_core::Ledger) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string_pretty(_ledger) {
                    let _ = storage.set_item(STORAGE_KEY, &json);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health() {
        let mut engine = WasmEngine::new();
        let res = engine.route("GET", "/api/health", None);
        assert_eq!(res.status, 200);
        assert_eq!(res.handler, "health");
        assert_eq!(res.data, serde_json::json!({ "ok": true }));
    }

    #[test]
    fn test_crud_and_summary() {
        let mut engine = WasmEngine::new();
        // Create
        let res = engine.route(
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"12.50","category":"food","note":"lunch","date":"2026-10-14"}"#),
        );
        assert_eq!(res.status, 201);
        assert_eq!(res.handler, "create_expense");
        assert_eq!(res.data["id"], 1);
        assert_eq!(res.data["amount"], 1250);
        assert_eq!(res.data["display"], "12.50");

        // List
        let res = engine.route("GET", "/api/expenses?category=food", None);
        assert_eq!(res.status, 200);
        assert_eq!(res.handler, "list_expenses");
        let list = res.data.as_array().expect("array");
        assert_eq!(list.len(), 1);

        // Summary
        let res = engine.route("GET", "/api/summary?month=2026-10", None);
        assert_eq!(res.status, 200);
        assert_eq!(res.handler, "summary");
        assert_eq!(res.data["total"]["display"], "12.50");

        // Set budget
        let res = engine.route(
            "PUT",
            "/api/budgets/food?month=2026-10",
            Some(r#"{"amount":"150.00"}"#),
        );
        assert_eq!(res.status, 200);
        assert_eq!(res.handler, "set_budget");
        assert_eq!(res.data["budget"]["display"], "150.00");
        assert_eq!(res.data["spent"]["display"], "12.50");
        assert_eq!(res.data["over"], false);

        // Delete
        let res = engine.route("DELETE", "/api/expenses/1", None);
        assert_eq!(res.status, 204);
        assert_eq!(res.handler, "delete_expense");

        // List after delete
        let res = engine.route("GET", "/api/expenses", None);
        assert_eq!(res.status, 200);
        assert_eq!(res.data.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_errors() {
        let mut engine = WasmEngine::new();
        let res = engine.route(
            "POST",
            "/api/expenses",
            Some(r#"{"amount":"abc","category":"food"}"#),
        );
        assert_eq!(res.status, 400);
        assert_eq!(res.handler, "create_expense");

        let res = engine.route("GET", "/api/expenses/999", None);
        assert_eq!(res.status, 404);
        assert_eq!(res.handler, "get_expense");
    }
}
