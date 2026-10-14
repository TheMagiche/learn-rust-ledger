//! Pure ledger logic. No files, no HTTP, no clock.
//!
//! LEARN(crates-workspace): this crate is `spend-core`. The workspace `Cargo.toml`
//! builds it together with `spend-store`, `spend-api`, and the `spend` binary so
//! they share one lockfile and can depend on each other by path.
//! LEARN(modules-visibility): `pub mod` exposes a module. Types stay private to
//! that module until they are also marked `pub`, which keeps the crate's surface small.

pub mod category;
pub mod error;
pub mod expense;
pub mod ledger;
pub mod money;
pub mod report;

pub use category::Category;
pub use error::SpendError;
pub use expense::{month_key, parse_user_date, parse_user_month, Expense, NewExpense};
pub use ledger::{BudgetStatus, Filter, Ledger};
pub use money::Money;
