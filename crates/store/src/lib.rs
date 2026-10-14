//! Persistence for a [`spend_core::Ledger`].
//!
//! LEARN(traits): `Store` is a contract with two methods. `JsonFileStore` and `MemoryStore`
//! both implement it, so the HTTP API can save without knowing which one it holds.

mod json_file;

pub use json_file::JsonFileStore;

use std::sync::Mutex;

use spend_core::Ledger;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid ledger json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ledger lock poisoned")]
    Poisoned,
}

pub trait Store: Send + Sync {
    fn load(&self) -> Result<Ledger, StoreError>;
    fn save(&self, ledger: &Ledger) -> Result<(), StoreError>;
}

/// In-memory store used by tests. It proves the API does not depend on the filesystem.
#[derive(Debug, Default)]
pub struct MemoryStore {
    inner: Mutex<Ledger>,
}

impl MemoryStore {
    pub fn new(ledger: Ledger) -> Self {
        Self {
            inner: Mutex::new(ledger),
        }
    }
}

impl Store for MemoryStore {
    fn load(&self) -> Result<Ledger, StoreError> {
        let guard = self.inner.lock().map_err(|_| StoreError::Poisoned)?;
        Ok(guard.clone())
    }

    fn save(&self, ledger: &Ledger) -> Result<(), StoreError> {
        let mut guard = self.inner.lock().map_err(|_| StoreError::Poisoned)?;
        *guard = ledger.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use spend_core::{Category, Money, NewExpense};

    use super::*;

    #[test]
    fn memory_store_round_trip() {
        let store = MemoryStore::default();
        let mut ledger = store.load().expect("load");
        ledger
            .add(NewExpense {
                amount: Money(250),
                category: Category::Fun,
                note: None,
                date: chrono::NaiveDate::from_ymd_opt(2026, 10, 14).expect("date"),
            })
            .expect("add");
        store.save(&ledger).expect("save");
        let loaded = store.load().expect("reload");
        assert_eq!(loaded.get(1).expect("row").amount, Money(250));
    }
}
