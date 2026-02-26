use std::{fs::File, io::BufReader, path::PathBuf};

use calamine::{Data, DataType, Reader, Xlsx, open_workbook};

use crate::{
    error::{ExcelDataReaderClientResult, ExcelDataReaderError},
    models::*,
};

pub struct ExcelDataReaderClient {
    excel_client: Xlsx<BufReader<File>>,
}

impl ExcelDataReaderClient {
    pub fn new(path: PathBuf) -> ExcelDataReaderClientResult<Self> {
        let excel_client = open_workbook(path)?;
        Ok(Self { excel_client })
    }

    pub fn base_client(&self) -> &Xlsx<BufReader<File>> {
        &self.excel_client
    }

    pub fn read_worksheet_data(
        &mut self,
        sheet_name: &str,
        headers: &[&str],
    ) -> ExcelDataReaderClientResult<Vec<Record>> {
        let range = self.excel_client.worksheet_range(sheet_name)?;

        let header_row = range
            .rows()
            .next()
            .ok_or(ExcelDataReaderError::EmptySheet)?;

        let time_col = find_column(header_row, headers[0])?;
        let qty_col = find_column(header_row, headers[1])?;

        let mut records = Vec::new();
        for row in range.rows().skip(1) {
            let time = row[time_col].as_datetime().ok_or_else(|| {
                ExcelDataReaderError::InvalidDate(format!(
                    "{:?}",
                    row[time_col]
                ))
            })?;

            let quantity = row[qty_col].get_float().ok_or_else(|| {
                ExcelDataReaderError::InvalidFloat(format!(
                    "{:?}",
                    row[qty_col]
                ))
            })?;

            records.push(Record { time, quantity });
        }

        Ok(records)
    }
}

fn find_column(
    header_row: &[Data],
    name: &str,
) -> ExcelDataReaderClientResult<usize> {
    header_row
        .iter()
        .position(|cell| cell.as_string().as_deref() == Some(name))
        .ok_or_else(|| ExcelDataReaderError::MissingHeader(name.to_string()))
}
