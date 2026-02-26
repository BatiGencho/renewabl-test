use uuid::Uuid;

use crate::wire_api::wire_error_v1::{WireV1Detail, WireV1Error};

pub type HandlerResult<T> = Result<T, WireV1Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Failed to get database connection: {0}")]
    PoolError(String),
}

impl Error {
    pub fn to_wire_v1_error(self, request_id: &Uuid) -> WireV1Error {
        match self {
            Error::DatabaseError(e) => WireV1Error::internal_server_error(
                "Aggregation query failed".to_string(),
                vec![WireV1Detail {
                    field: None,
                    code: "database_error".to_string(),
                    message: format!("Database error: {e}"),
                    suggestion: "Please try again later".to_string(),
                    documentation: String::new(),
                }],
                request_id.to_string(),
            ),
            Error::PoolError(e) => WireV1Error::service_unavailable(
                "Service temporarily unavailable".to_string(),
                vec![WireV1Detail {
                    field: None,
                    code: "pool_error".to_string(),
                    message: format!("Failed to get database connection: {e}"),
                    suggestion: "Please try again later".to_string(),
                    documentation: String::new(),
                }],
                request_id.to_string(),
            ),
        }
    }
}

impl crate::wire_api::error_recorder::IntoWireV1Error for Error {
    fn into_wire_v1_error(self, request_id: &Uuid) -> WireV1Error {
        self.to_wire_v1_error(request_id)
    }
}
