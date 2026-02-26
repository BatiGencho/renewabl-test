// The OpenApi derive macro generates code using Iterator::for_each,
// which is disallowed by our clippy config. Allow it at module level.
#![allow(clippy::disallowed_methods)]

use utoipa::OpenApi;

/// Main OpenAPI documentation for the Wire v1 API
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::wire_api::core::v1::energy::aggregate::handler::handler,
        crate::wire_api::core::v1::energy::history::handler::handler,
    ),
    info(
        title = "Energy Readings API",
        version = "1.0.0",
        description = "REST API for querying timeseries energy data with aggregation and date filters",
        license(name = "MIT")
    ),
    servers(
        (url = "/api/wire/v1", description = "API v1")
    ),
    tags(
        (name = "energy", description = "Energy readings aggregation and query history")
    )
)]
pub struct WireV1ApiDoc;

impl WireV1ApiDoc {
    pub fn openapi() -> utoipa::openapi::OpenApi {
        let openapi = <WireV1ApiDoc as utoipa::OpenApi>::openapi();
        openapi
    }

    /// Get OpenAPI spec as fixed JSON for OpenAPI 3.0 compatibility
    /// Converts type: ["array", "null"] to type: "array", nullable: true
    pub fn openapi_json() -> serde_json::Value {
        let openapi = Self::openapi();

        // Serialize to JSON
        let mut json_value = serde_json::to_value(&openapi)
            .expect("Failed to serialize OpenAPI spec");

        // Recursively fix all type: ["array", "null"] patterns
        let fixed_count = Self::fix_nullable_arrays_recursive(&mut json_value);

        if fixed_count > 0 {
            tracing::info!(
                "Fixed {} nullable array type definitions in OpenAPI spec",
                fixed_count
            );
        }

        json_value
    }

    fn fix_nullable_arrays_recursive(value: &mut serde_json::Value) -> usize {
        let mut fixed_count = 0;

        match value {
            serde_json::Value::Object(map) => {
                // Check if this object has the problematic type pattern
                if let Some(type_value) = map.get("type")
                    && let serde_json::Value::Array(type_array) = type_value
                {
                    // Check if it's ["array", "null"] or ["null", "array"]
                    let has_array = type_array.iter().any(|v| v == "array");
                    let has_null = type_array.iter().any(|v| v == "null");

                    if has_array && has_null && type_array.len() == 2 {
                        // Fix it: set type to "array" and add nullable: true
                        map.insert(
                            "type".to_string(),
                            serde_json::Value::String("array".to_string()),
                        );
                        map.insert(
                            "nullable".to_string(),
                            serde_json::Value::Bool(true),
                        );
                        fixed_count += 1;
                    }
                }

                // Recursively process all values in the object
                for (_key, val) in map.iter_mut() {
                    fixed_count += Self::fix_nullable_arrays_recursive(val);
                }
            }
            serde_json::Value::Array(arr) => {
                // Recursively process all items in the array
                for item in arr.iter_mut() {
                    fixed_count += Self::fix_nullable_arrays_recursive(item);
                }
            }
            _ => {}
        }

        fixed_count
    }
}
