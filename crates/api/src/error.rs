//! HTTP errors. Spend failures become 400 or 404 JSON.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use spend_core::SpendError;
use spend_store::StoreError;

#[derive(Debug)]
pub enum ApiError {
    Spend(SpendError),
    BadRequest(String),
    Store(StoreError),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ApiError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ApiError::Spend(SpendError::NotFound(_)) => (StatusCode::NOT_FOUND, self.message()),
            ApiError::Spend(_) | ApiError::BadRequest(_) => {
                (StatusCode::BAD_REQUEST, self.message())
            }
            ApiError::Store(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.message()),
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::Spend(err) => err.to_string(),
            ApiError::BadRequest(message) => message.clone(),
            ApiError::Store(err) => err.to_string(),
        }
    }
}

/// LEARN(from-into): `From<SpendError>` lets `?` turn a ledger error into an `ApiError`
/// without an explicit `map_err` at every call.
impl From<SpendError> for ApiError {
    fn from(value: SpendError) -> Self {
        ApiError::Spend(value)
    }
}

impl From<StoreError> for ApiError {
    fn from(value: StoreError) -> Self {
        ApiError::Store(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = self.status_and_message();
        (status, Json(ErrorBody { error: message })).into_response()
    }
}
