use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

#[derive(Queryable, Selectable, Debug, Clone, serde::Serialize)]
#[diesel(table_name = crate::schema::query_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QueryHistory {
    pub id: Uuid,
    pub aggregation_type: String,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::query_history)]
pub struct NewQueryHistory {
    pub aggregation_type: String,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
}

impl QueryHistory {
    pub async fn create(
        entry: NewQueryHistory,
        conn: &mut AsyncPgConnection,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::query_history::dsl::*;

        diesel::insert_into(query_history)
            .values(&entry)
            .returning(QueryHistory::as_returning())
            .get_result(conn)
            .await
    }

    /// Get the last N query history entries ordered by most recent first.
    pub async fn get_latest(
        limit: i64,
        conn: &mut AsyncPgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::query_history::dsl::*;

        query_history
            .order(created_at.desc())
            .limit(limit)
            .load(conn)
            .await
    }
}
