use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use postgres_models::connection::{WithConnectionError, with_connection};
use postgres_models::models::query_history::QueryHistory;

use crate::AppState;
use crate::shared::extractors::request_id::RequestId;
use crate::wire_api::error_recorder::ErrorRecorder;

use super::errors::{self, HandlerResult};
use super::models::{HistoryResponse, QueryHistoryEntry};

const HANDLER_NAME: &str = "energy_history";
const HISTORY_LIMIT: i64 = 10;

/// Get the last 10 aggregation queries
///
/// Returns the most recent query history entries with their filter parameters.
#[utoipa::path(
    get,
    path = "/energy/history",
    responses(
        (status = 200, description = "Last 10 queries", body = HistoryResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "energy",
)]
#[tracing::instrument(skip_all, name = "energy_history")]
pub async fn handler(
    State(state): State<AppState>,
    RequestId(request_id): RequestId,
) -> HandlerResult<(StatusCode, Json<HistoryResponse>)> {
    let recorder =
        ErrorRecorder::new(&state.telemetry, HANDLER_NAME, &request_id);

    let entries =
        with_connection(&state.read_only_pool, |mut conn| async move {
            QueryHistory::get_latest(HISTORY_LIMIT, &mut conn).await
        })
        .await
        .map_err(|e| match e {
            WithConnectionError::Pool(e) => recorder
                .record("pool_error", errors::Error::PoolError(e.to_string())),
            WithConnectionError::Operation(e) => recorder
                .record("database_error", errors::Error::DatabaseError(e)),
        })?;

    let queries = entries
        .into_iter()
        .map(|e| QueryHistoryEntry {
            id: e.id,
            aggregation_type: e.aggregation_type,
            date_from: e.date_from,
            date_to: e.date_to,
            created_at: e.created_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(HistoryResponse { queries })))
}
