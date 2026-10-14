//! Request handlers. Each one is named in the `X-Rust-Handler` header.

use std::str::FromStr;

use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use spend_core::{
    month_key, parse_user_date, parse_user_month, BudgetStatus, Category, Expense, Filter, Money,
    NewExpense, SpendError,
};

use crate::error::ApiError;
use crate::state::SharedState;

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

#[derive(Debug, Deserialize)]
pub struct ExpenseQuery {
    category: Option<String>,
    month: Option<String>,
    search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MonthQuery {
    month: Option<String>,
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
    chrono::Local::now().date_naive()
}

fn current_month() -> String {
    month_key(today())
}

/// LEARN(async-await): `async fn` returns a future. Axum awaits it on the tokio runtime.
pub async fn health() -> Json<HealthBody> {
    Json(HealthBody { ok: true })
}

pub async fn list_expenses(
    State(state): State<SharedState>,
    query: Result<Query<ExpenseQuery>, QueryRejection>,
) -> Result<Json<Vec<ExpenseJson>>, ApiError> {
    let Query(query) =
        query.map_err(|err| ApiError::BadRequest(format!("invalid query: {err}")))?;
    let filter = filter_from_query(&query)?;
    // LEARN(borrowing): the mutex guard is a borrow of the ledger. `list` only needs `&self`.
    let ledger = state.ledger.lock().await;
    let rows = ledger.list(&filter).into_iter().map(expense_json).collect();
    Ok(Json(rows))
}

pub async fn create_expense(
    State(state): State<SharedState>,
    body: Result<Json<CreateExpenseBody>, JsonRejection>,
) -> Result<(StatusCode, Json<ExpenseJson>), ApiError> {
    // LEARN(question-mark): `?` returns the error early. `From` turns `SpendError` into `ApiError`.
    let Json(body) =
        body.map_err(|err| ApiError::BadRequest(format!("invalid JSON body: {err}")))?;
    // LEARN(integer-money): the body sends "12.50", not a JSON float. We parse it into cents.
    // LEARN(newtype): `Money::parse` returns `Money`, which cannot be confused with a raw `u64`.
    let amount = Money::parse(&body.amount)?;
    // LEARN(structs): `NewExpense` groups the fields `Ledger::add` needs before it assigns an id.
    let draft = NewExpense {
        amount,
        category: Category::from_str(body.category.trim())?,
        note: body.note,
        date: match body.date {
            Some(value) => parse_user_date(&value)?,
            None => today(),
        },
    };

    let mut ledger = state.ledger.lock().await;
    let backup = ledger.clone();
    // LEARN(ownership): `draft` moves into `add`. The ledger stores an `Expense`; we clone it
    // so we can still serialize the new row after the borrow from `add` ends.
    // LEARN(mut-borrow): `ledger` is `&mut Ledger` because `add` changes the vector.
    let created = {
        let expense = ledger.add(draft)?;
        expense.clone()
    };
    if let Err(err) = state.store.save(&ledger) {
        *ledger = backup;
        return Err(ApiError::Store(err));
    }
    // LEARN(result): `Ok` is the success path of `Result`. Axum turns this into HTTP 201.
    Ok((StatusCode::CREATED, Json(expense_json(&created))))
}

pub async fn get_expense(
    State(state): State<SharedState>,
    path: Result<Path<u32>, PathRejection>,
) -> Result<Json<ExpenseJson>, ApiError> {
    let Path(id) = path.map_err(|_| ApiError::BadRequest("invalid expense id".to_string()))?;
    let ledger = state.ledger.lock().await;
    // LEARN(option): `get` returns `Option`. `ok_or` turns `None` into a `NotFound` error.
    let expense = ledger.get(id).ok_or(SpendError::NotFound(id))?;
    Ok(Json(expense_json(expense)))
}

pub async fn delete_expense(
    State(state): State<SharedState>,
    path: Result<Path<u32>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(id) = path.map_err(|_| ApiError::BadRequest("invalid expense id".to_string()))?;
    let mut ledger = state.ledger.lock().await;
    let backup = ledger.clone();
    // LEARN(move): `remove` gives us the owned `Expense`. The vector no longer holds it.
    let removed = ledger.remove(id)?;
    tracing::debug!(id = removed.id, "removed expense");
    if let Err(err) = state.store.save(&ledger) {
        *ledger = backup;
        return Err(ApiError::Store(err));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn summary(
    State(state): State<SharedState>,
    query: Result<Query<MonthQuery>, QueryRejection>,
) -> Result<Json<SummaryJson>, ApiError> {
    let Query(query) =
        query.map_err(|err| ApiError::BadRequest(format!("invalid query: {err}")))?;
    let month = match query.month {
        Some(value) if !value.trim().is_empty() => parse_user_month(&value)?,
        _ => current_month(),
    };
    let ledger = state.ledger.lock().await;
    let filter = Filter {
        month: Some(month.clone()),
        ..Filter::default()
    };
    let total = ledger.total(&filter);
    // LEARN(btreemap): `by_category` returns a sorted map, so the chart order is stable.
    let by_category = ledger
        .by_category(Some(&month))
        .into_iter()
        .map(|(category, money)| CategoryTotalJson {
            category,
            amount: money.cents(),
            display: money.to_string(),
        })
        .collect();
    // LEARN(hashmap): budgets were stored in a `HashMap` and are read back here.
    let budgets = ledger
        .budget_status(&month)
        .iter()
        .map(budget_json)
        .collect();
    Ok(Json(SummaryJson {
        total: money_json(total),
        by_category,
        budgets,
    }))
}

pub async fn set_budget(
    State(state): State<SharedState>,
    Path(category_text): Path<String>,
    Query(query): Query<MonthQuery>,
    body: Result<Json<AmountBody>, JsonRejection>,
) -> Result<Json<BudgetJson>, ApiError> {
    let Json(body) =
        body.map_err(|err| ApiError::BadRequest(format!("invalid JSON body: {err}")))?;
    let amount = Money::parse(&body.amount)?;
    let category = Category::from_str(category_text.trim())?;
    let month = match query.month {
        Some(value) if !value.trim().is_empty() => parse_user_month(&value)?,
        _ => current_month(),
    };
    let mut ledger = state.ledger.lock().await;
    let backup = ledger.clone();
    ledger.set_budget(category, amount);
    if let Err(err) = state.store.save(&ledger) {
        *ledger = backup;
        return Err(ApiError::Store(err));
    }
    let status = ledger
        .budget_status(&month)
        .into_iter()
        .find(|row| row.category == category)
        .ok_or(SpendError::UnknownCategory(category.to_string()))?;
    Ok(Json(budget_json(&status)))
}

fn filter_from_query(query: &ExpenseQuery) -> Result<Filter, ApiError> {
    let category = match &query.category {
        Some(value) if !value.trim().is_empty() => Some(Category::from_str(value)?),
        _ => None,
    };
    let month = match &query.month {
        Some(value) if !value.trim().is_empty() => Some(parse_user_month(value)?),
        _ => None,
    };
    let search = query
        .search
        .clone()
        .filter(|value| !value.trim().is_empty());
    Ok(Filter {
        category,
        month,
        search,
    })
}
