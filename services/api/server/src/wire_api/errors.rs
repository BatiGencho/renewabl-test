use crate::shared::errors::{ApiError, ErrorDetail};
use crate::wire_api::wire_error::{Detail, WireError};
use axum::http::StatusCode;

/// Compatibility layer for Wire API - converts unified ApiError to WireError format
impl From<ApiError> for WireError {
    fn from(api_error: ApiError) -> Self {
        let details: Vec<Detail> = api_error
            .details
            .into_iter()
            .map(|detail| Detail {
                field: detail.field.unwrap_or_default(),
                code: detail.code.to_string(),
                message: detail.message,
            })
            .collect();

        WireError {
            status_code: StatusCode::from_u16(api_error.status_code)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            message: api_error.message,
            details,
            timestamp: api_error.timestamp,
        }
    }
}

/// Conversion from WireError to unified ApiError
impl From<WireError> for ApiError {
    fn from(wire_error: WireError) -> Self {
        let details: Vec<ErrorDetail> = wire_error
            .details
            .into_iter()
            .map(|detail| ErrorDetail {
                field: if detail.field.is_empty() {
                    None
                } else {
                    Some(detail.field)
                },
                code: Box::leak(detail.code.into_boxed_str()), // Convert String to &'static str for compatibility
                message: detail.message,
            })
            .collect();

        ApiError {
            status_code: wire_error.status_code.as_u16(),
            code: "WIREERROR",
            message: wire_error.message,
            details,
            timestamp: wire_error.timestamp,
            context: None,
        }
    }
}
