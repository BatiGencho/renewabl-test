use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AggregationType {
    Hourly,
    DayOfMonth,
    Monthly,
}

impl AggregationType {
    pub fn to_trunc_level(&self) -> &str {
        match self {
            AggregationType::Hourly => "hour",
            AggregationType::DayOfMonth => "day",
            AggregationType::Monthly => "month",
        }
    }
}

impl std::fmt::Display for AggregationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregationType::Hourly => write!(f, "hourly"),
            AggregationType::DayOfMonth => write!(f, "day_of_month"),
            AggregationType::Monthly => write!(f, "monthly"),
        }
    }
}

/// Request payload for aggregating energy readings
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AggregateRequest {
    /// Aggregation granularity
    #[schema(example = "monthly")]
    pub aggregation_type: AggregationType,

    /// Start of date range (inclusive, optional)
    #[schema(example = "2025-01-01T00:00:00Z")]
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,

    /// End of date range (exclusive, optional)
    #[schema(example = "2025-04-01T00:00:00Z")]
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
}

/// A single aggregated data point
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AggregateDataPoint {
    /// Start of the aggregation period
    #[schema(example = "2025-01-01T00:00:00Z")]
    pub period: chrono::DateTime<chrono::Utc>,

    /// Total energy in kWh for this period
    #[schema(example = "216000.0000")]
    pub total_kwh: String,
}

/// Response for an aggregation query
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AggregateResponse {
    pub aggregation_type: AggregationType,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
    pub data: Vec<AggregateDataPoint>,
}
