#[derive(Debug, Clone)]
pub struct Record {
    pub time: chrono::NaiveDateTime,
    pub quantity: f64,
}
