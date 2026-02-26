use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Numeric, Timestamptz};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

#[derive(Queryable, Selectable, Debug, Clone, serde::Serialize)]
#[diesel(table_name = crate::schema::energy_readings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EnergyReading {
    pub id: Uuid,
    pub reading_time: DateTime<Utc>,
    pub quantity_kwh: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::energy_readings)]
pub struct NewEnergyReading {
    pub reading_time: DateTime<Utc>,
    pub quantity_kwh: BigDecimal,
}

#[derive(QueryableByName, Debug, Clone, serde::Serialize)]
pub struct AggregatedReading {
    #[diesel(sql_type = Timestamptz)]
    pub period: DateTime<Utc>,
    #[diesel(sql_type = Numeric)]
    pub total_kwh: BigDecimal,
}

impl EnergyReading {
    /// Bulk insert energy readings - skipping conflicts on reading_time (upsert).
    pub async fn bulk_insert(
        readings: Vec<NewEnergyReading>,
        conn: &mut AsyncPgConnection,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::energy_readings::dsl::*;

        diesel::insert_into(energy_readings)
            .values(&readings)
            .on_conflict(reading_time)
            .do_nothing()
            .execute(conn)
            .await
    }

    /// Count total rows in the table.
    pub async fn count(
        conn: &mut AsyncPgConnection,
    ) -> Result<i64, diesel::result::Error> {
        use crate::schema::energy_readings::dsl::*;

        energy_readings.count().get_result(conn).await
    }

    /// Aggregate energy readings by the given truncation level (hour, day, month).
    pub async fn aggregate(
        trunc_level: &str,
        date_from: Option<DateTime<Utc>>,
        date_to: Option<DateTime<Utc>>,
        conn: &mut AsyncPgConnection,
    ) -> Result<Vec<AggregatedReading>, diesel::result::Error> {
        let mut query = String::from(
            "SELECT date_trunc($1, reading_time) AS period, \
             SUM(quantity_kwh) AS total_kwh \
             FROM energy_readings WHERE 1=1",
        );

        // Build parameter list dynamically
        // $1 = trunc_level (always present)
        // $2 = date_from (if present)
        // $3 = date_to (if present)
        let mut param_idx = 2;

        if date_from.is_some() {
            query.push_str(&format!(" AND reading_time >= ${param_idx}"));
            param_idx += 1;
        }
        if date_to.is_some() {
            query.push_str(&format!(" AND reading_time < ${param_idx}"));
        }

        query.push_str(" GROUP BY period ORDER BY period");

        // apply the correc ind params
        match (date_from, date_to) {
            (Some(from), Some(to)) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(trunc_level)
                    .bind::<Timestamptz, _>(from)
                    .bind::<Timestamptz, _>(to)
                    .load::<AggregatedReading>(conn)
                    .await
            }
            (Some(from), None) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(trunc_level)
                    .bind::<Timestamptz, _>(from)
                    .load::<AggregatedReading>(conn)
                    .await
            }
            (None, Some(to)) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(trunc_level)
                    .bind::<Timestamptz, _>(to)
                    .load::<AggregatedReading>(conn)
                    .await
            }
            (None, None) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(trunc_level)
                    .load::<AggregatedReading>(conn)
                    .await
            }
        }
    }
}
