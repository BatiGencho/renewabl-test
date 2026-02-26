use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use deadpool_redis::redis::AsyncCommands;
use postgres_models::connection::{WithConnectionError, with_connection};
use postgres_models::models::energy_readings::EnergyReading;
use postgres_models::models::query_history::{NewQueryHistory, QueryHistory};

use crate::AppState;
use crate::shared::extractors::request_id::RequestId;
use crate::shared::extractors::validations::ValidatedPayload;
use crate::wire_api::error_recorder::ErrorRecorder;

use super::errors::{self, HandlerResult};
use super::models::{AggregateDataPoint, AggregateRequest, AggregateResponse};

const HANDLER_NAME: &str = "energy_aggregate";
const CACHE_TTL_SECONDS: u64 = 300; // 5 minutes

fn cache_key(payload: &AggregateRequest) -> String {
    format!(
        "energy:aggregate:{}:{}:{}",
        payload.aggregation_type,
        payload
            .date_from
            .map_or("none".to_string(), |d| d.to_rfc3339()),
        payload
            .date_to
            .map_or("none".to_string(), |d| d.to_rfc3339()),
    )
}

/// Aggregate energy readings by hour, day, or month
///
/// Returns energy consumption summed by the requested granularity,
/// optionally filtered by date range.
#[utoipa::path(
    post,
    path = "/energy/aggregate",
    request_body = AggregateRequest,
    responses(
        (status = 200, description = "Aggregated energy data", body = AggregateResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "energy",
)]
#[tracing::instrument(skip_all, name = "energy_aggregate")]
pub async fn handler(
    State(state): State<AppState>,
    RequestId(request_id): RequestId,
    ValidatedPayload(payload): ValidatedPayload<AggregateRequest>,
) -> HandlerResult<(StatusCode, Json<AggregateResponse>)> {
    tracing::info!(
        aggregation_type = %payload.aggregation_type,
        date_from = ?payload.date_from,
        date_to = ?payload.date_to,
        request_id = %request_id,
        "Energy aggregate request",
    );

    let recorder =
        ErrorRecorder::new(&state.telemetry, HANDLER_NAME, &request_id);

    let new_entry = NewQueryHistory {
        aggregation_type: payload.aggregation_type.to_string(),
        date_from: payload.date_from,
        date_to: payload.date_to,
    };
    with_connection(&state.pool, |mut conn| async move {
        QueryHistory::create(new_entry, &mut conn).await
    })
    .await
    .map_err(|e| match e {
        WithConnectionError::Pool(e) => recorder
            .record("pool_error", errors::Error::PoolError(e.to_string())),
        WithConnectionError::Operation(e) => {
            recorder.record("database_error", errors::Error::DatabaseError(e))
        }
    })?;

    let key = cache_key(&payload);
    if let Ok(mut conn) = state.cache_pool.get().await {
        let cached: Result<Option<String>, _> = conn.get(&key).await;
        if let Ok(Some(json_str)) = cached {
            if let Ok(response) =
                serde_json::from_str::<AggregateResponse>(&json_str)
            {
                tracing::debug!("Cache hit for {key}");
                return Ok((StatusCode::OK, Json(response)));
            }
        }
    }

    let trunc_level = payload.aggregation_type.to_trunc_level().to_owned();
    let date_from = payload.date_from;
    let date_to = payload.date_to;

    let rows = with_connection(&state.read_only_pool, |mut conn| async move {
        EnergyReading::aggregate(&trunc_level, date_from, date_to, &mut conn)
            .await
    })
    .await
    .map_err(|e| match e {
        WithConnectionError::Pool(e) => recorder
            .record("pool_error", errors::Error::PoolError(e.to_string())),
        WithConnectionError::Operation(e) => {
            recorder.record("database_error", errors::Error::DatabaseError(e))
        }
    })?;

    let data = rows
        .into_iter()
        .map(|r| AggregateDataPoint {
            period: r.period,
            total_kwh: r.total_kwh.to_string(),
        })
        .collect();

    let response = AggregateResponse {
        aggregation_type: payload.aggregation_type,
        date_from: payload.date_from,
        date_to: payload.date_to,
        data,
    };

    if let Ok(json_str) = serde_json::to_string(&response) {
        if let Ok(mut conn) = state.cache_pool.get().await {
            let _: Result<(), _> =
                conn.set_ex(&key, &json_str, CACHE_TTL_SECONDS).await;
        }
    }

    Ok((StatusCode::OK, Json(response)))
}
