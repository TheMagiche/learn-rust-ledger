//! Shared ledger for every request.

use std::sync::Arc;

use spend_core::Ledger;
use spend_store::Store;

pub struct AppState {
    /// LEARN(mutex): tokio's `Mutex` lets one task at a time change the ledger.
    /// `.lock().await` waits without blocking the runtime thread. This mutex does not poison.
    pub ledger: tokio::sync::Mutex<Ledger>,
    /// LEARN(trait-objects): `dyn Store` is a trait object. The concrete type is either
    /// `JsonFileStore` or `MemoryStore`, and this box owns it.
    /// `Send + Sync` is required so the box can move across threads and be shared.
    pub store: Box<dyn Store + Send + Sync>,
}

/// LEARN(arc): `Arc` counts references. Axum clones this pointer into each handler.
/// Cloning the `Arc` does not clone the ledger.
pub type SharedState = Arc<AppState>;
