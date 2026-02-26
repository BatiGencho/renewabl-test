use std::error::Error;

use crate::secrets::get_secret;

pub struct DatabaseConnections {
    pub postgres: postgres_models::connection::Pool,
    pub redis: redis_cache::connection::Pool,
}

pub async fn establish_connections()
-> Result<DatabaseConnections, Box<dyn Error>> {
    let db_rw_url = get_database_url().await?;
    let redis_url = get_secret("REDIS_URL").await?;

    let postgres = postgres_models::connection::establish_connection(db_rw_url)
        .await
        .expect("failed to connect to Postgres");

    let redis = redis_cache::connection::establish_connection(redis_url)
        .await
        .expect("failed to connect to Redis");

    Ok(DatabaseConnections { postgres, redis })
}

pub async fn get_redis_connection()
-> Result<redis_cache::connection::Pool, Box<dyn Error>> {
    let redis_url = get_secret("REDIS_URL").await?;
    let redis = redis_cache::connection::establish_connection(redis_url)
        .await
        .expect("failed to connect to Redis");

    Ok(redis)
}

async fn get_database_url() -> Result<String, Box<dyn Error>> {
    match get_secret("DATABASE_URL").await {
        Ok(url) => Ok(url),
        Err(_) => {
            let database_credentials_string =
                get_secret("DATABASE_CREDENTIALS").await?;
            let database_credentials = serde_json::from_str::<
                postgres_models::connection::Credentials,
            >(
                database_credentials_string.as_str()
            )
            .expect("DATABASE_CREDENTIALS must be valid");

            let db_username = database_credentials.username;
            let db_password = database_credentials.password;
            let db_rw_endpoint = get_secret("DATABASE_RW_ENDPOINT").await?;
            let db_rw_url = format!(
                "postgresql://{db_username}:{db_password}@{db_rw_endpoint}:5432/wire"
            );

            Ok(db_rw_url)
        }
    }
}
