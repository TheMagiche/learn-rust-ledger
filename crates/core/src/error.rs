//! Failures a caller can fix by changing the amount, category, date, or id.

use thiserror::Error;

/// LEARN(custom-errors): one enum lists every way ledger logic can fail.
/// `thiserror` writes the `Display` text and the `Error` trait for us.
/// Callers match on the variant when they care, or print it when they do not.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SpendError {
    #[error("amount must be greater than zero")]
    ZeroAmount,
    #[error("invalid amount: {0}")]
    BadAmount(String),
    #[error("unknown category: {0}")]
    UnknownCategory(String),
    #[error("expense {0} not found")]
    NotFound(u32),
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("invalid month: {0}")]
    InvalidMonth(String),
}
