//! One recorded purchase, plus the date strings the CLI and API share.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::category::Category;
use crate::error::SpendError;
use crate::money::Money;

/// LEARN(structs): a struct names its fields. An expense is an id, an amount, a category, an optional note, and a date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expense {
    pub id: u32,
    pub amount: Money,
    /// LEARN(serde): `Serialize` and `Deserialize` turn this struct into JSON and back.
    /// `Money` is a newtype, so the file stores a bare integer of minor units.
    pub category: Category,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub date: NaiveDate,
}

/// Fields the ledger needs before it assigns an id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewExpense {
    pub amount: Money,
    pub category: Category,
    pub note: Option<String>,
    pub date: NaiveDate,
}

/// `YYYY-MM-DD`. Anything else is [`SpendError::InvalidDate`].
pub fn parse_user_date(text: &str) -> Result<NaiveDate, SpendError> {
    let text = text.trim();
    // LEARN(slices): `as_bytes()` gives a `&[u8]` slice. `bytes[4]` and `bytes[7]` look at
    // single positions without copying the string.
    let bytes = text.as_bytes();
    if bytes.len() == 10 && bytes[4] == b'-' && bytes[7] == b'-' {
        if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
            return Ok(date);
        }
    }
    Err(SpendError::InvalidDate(text.to_string()))
}

/// `YYYY-MM`. The returned string is the canonical month key.
pub fn parse_user_month(text: &str) -> Result<String, SpendError> {
    let text = text.trim();
    let bytes = text.as_bytes();
    let shape_ok = bytes.len() == 7
        && bytes[4] == b'-'
        && bytes[..4].iter().all(|byte| byte.is_ascii_digit())
        && bytes[5..].iter().all(|byte| byte.is_ascii_digit());
    if shape_ok && NaiveDate::parse_from_str(&format!("{text}-01"), "%Y-%m-%d").is_ok() {
        return Ok(text.to_string());
    }
    Err(SpendError::InvalidMonth(text.to_string()))
}

pub fn month_key(date: NaiveDate) -> String {
    format!("{:04}-{:02}", date.year(), date.month())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_months() {
        assert!(parse_user_date("2026-10-14").is_ok());
        assert!(parse_user_date("2026-13-01").is_err());
        assert!(parse_user_date("10/14/2026").is_err());
        assert_eq!(parse_user_month("2026-10").expect("month"), "2026-10");
        assert!(parse_user_month("2026-13").is_err());
        assert!(parse_user_month("2026-1").is_err());
    }
}
