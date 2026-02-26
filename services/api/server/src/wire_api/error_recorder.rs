use std::sync::Arc;

use telemetry::metrics::Telemetry;
use uuid::Uuid;

use crate::metrics::ServerMetrics;
use crate::wire_api::wire_error_v1::WireV1Error;

/// Trait for handler error types that can be converted to [`WireV1Error`].
pub trait IntoWireV1Error {
    fn into_wire_v1_error(self, request_id: &Uuid) -> WireV1Error;
}

/// Records error metrics and converts handler errors to [`WireV1Error`].
///
/// Replaces the per-handler `record_err` closures with a single reusable type.
pub struct ErrorRecorder<'a> {
    telemetry: &'a Arc<Telemetry<ServerMetrics>>,
    handler_name: &'a str,
    request_id: &'a Uuid,
}

impl<'a> ErrorRecorder<'a> {
    pub fn new(
        telemetry: &'a Arc<Telemetry<ServerMetrics>>,
        handler_name: &'a str,
        request_id: &'a Uuid,
    ) -> Self {
        Self {
            telemetry,
            handler_name,
            request_id,
        }
    }

    pub fn record<E: IntoWireV1Error>(&self, code: &str, e: E) -> WireV1Error {
        self.telemetry.maybe_use_metrics(|m| {
            m.record_error(self.handler_name, code);
        });
        e.into_wire_v1_error(self.request_id)
    }
}
