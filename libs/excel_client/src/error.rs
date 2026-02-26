use calamine::XlsxError;
use thiserror::Error;

pub type ExcelDataReaderClientResult<T> = Result<T, ExcelDataReaderError>;

#[derive(Error, Debug)]
pub enum ExcelDataReaderError {
    #[error("Calamine (xlsx reader) error: {0}")]
    Xlsx(#[from] XlsxError),

    #[error("Sheet is empty, no header row found")]
    EmptySheet,

    #[error("Header column not found: {0}")]
    MissingHeader(String),

    #[error("Cell is not a valid date: {0}")]
    InvalidDate(String),

    #[error("Cell is not a valid number: {0}")]
    InvalidFloat(String),
}
