// @generated automatically by Diesel CLI.

diesel::table! {
    energy_readings (id) {
        id -> Uuid,
        reading_time -> Timestamptz,
        quantity_kwh -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    query_history (id) {
        id -> Uuid,
        aggregation_type -> Text,
        date_from -> Nullable<Timestamptz>,
        date_to -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(energy_readings, query_history,);
