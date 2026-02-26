use bigdecimal::BigDecimal;
use chrono::{TimeZone, Utc};
use postgres_models::models::energy_readings::{
    EnergyReading, NewEnergyReading,
};
use std::path::PathBuf;
use std::str::FromStr;

const SHEET_NAME: &str = "Sheet1";
const HEADERS: &[&str] = &["Time (UTC)", "Quantity kWh"];
const BATCH_SIZE: usize = 1000;

pub async fn load_energy_readings(
    file_path: &str,
    pool: &postgres_models::connection::Pool,
) -> anyhow::Result<()> {
    let mut conn = pool.get().await.map_err(|e| {
        anyhow::anyhow!("Failed to get DB connection for data loading: {e}")
    })?;

    let existing_count = EnergyReading::count(&mut conn).await?;
    if existing_count > 0 {
        tracing::info!(
            count = existing_count,
            "Energy readings already loaded, skipping import"
        );
        return Ok(());
    }

    tracing::info!(file = %file_path, "Loading energy readings from Excel");

    let path = PathBuf::from(file_path);
    let mut client = excel_client::ExcelDataReaderClient::new(path)?;
    let records = client.read_worksheet_data(SHEET_NAME, HEADERS)?;

    tracing::info!(records = records.len(), "Parsed records from Excel");

    let mut new_readings = Vec::with_capacity(records.len());
    for record in &records {
        let reading_time = Utc.from_utc_datetime(&record.time);
        let quantity_kwh = BigDecimal::from_str(&format!(
            "{:.4}",
            record.quantity
        ))
        .map_err(|e| {
            anyhow::anyhow!("Invalid quantity '{}': {e}", record.quantity)
        })?;

        new_readings.push(NewEnergyReading {
            reading_time,
            quantity_kwh,
        });
    }

    let mut total_inserted = 0usize;
    for chunk in new_readings.chunks(BATCH_SIZE) {
        let inserted =
            EnergyReading::bulk_insert(chunk.to_vec(), &mut conn).await?;
        total_inserted += inserted;
    }

    tracing::info!(
        inserted = total_inserted,
        total = new_readings.len(),
        "Energy readings loaded into database"
    );

    Ok(())
}
