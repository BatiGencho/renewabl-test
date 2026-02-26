#[cfg(test)]
mod tests {
    use crate::client::ExcelDataReaderClient;
    use std::path::PathBuf;

    #[test]
    fn test_client_custom_base_url() {
        let mut client = ExcelDataReaderClient::new(PathBuf::from(
            "../../Test January2025-December2025-hourly-example.xlsx",
        ))
        .unwrap();
        let entries = client
            .read_worksheet_data("Sheet1", &["Time (UTC)", "Quantity kWh"])
            .unwrap();
        println!("entries={:?}", entries);
        assert!(entries.len() > 0);
    }
}
