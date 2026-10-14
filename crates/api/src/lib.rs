//! HTTP API. Handlers borrow the ledger, then the JSON store writes it back.
//!
//! LEARN(tokio): `serve` binds a socket on the tokio runtime and waits for Ctrl-C
//! before shutting down. In-flight requests finish; new ones stop being accepted.

mod error;
mod handlers;
mod learn;
mod routes;
mod state;

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub use error::ApiError;
pub use routes::router;
pub use state::{AppState, SharedState};

use spend_store::{Store, StoreError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServeError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub async fn serve(
    addr: &str,
    store: Box<dyn Store + Send + Sync>,
    dev: bool,
) -> Result<(), ServeError> {
    let ledger = store.load()?;
    let state = Arc::new(AppState {
        ledger: Mutex::new(ledger),
        store,
    });
    let app = router(state, dev);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
