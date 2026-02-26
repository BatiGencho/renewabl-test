use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EnergyType {
    Solar,
    Wind,
    Hydro,
    Geothermal,
    Biomass,
    Tidal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlantStatus {
    Active,
    Inactive,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plant {
    pub id: Uuid,
    pub name: String,
    pub energy_type: EnergyType,
    pub capacity_mw: f64,
    pub location: String,
    pub status: PlantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlantRequest {
    pub name: String,
    pub energy_type: EnergyType,
    pub capacity_mw: f64,
    pub location: String,
    pub status: Option<PlantStatus>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlantRequest {
    pub name: Option<String>,
    pub energy_type: Option<EnergyType>,
    pub capacity_mw: Option<f64>,
    pub location: Option<String>,
    pub status: Option<PlantStatus>,
}

impl Plant {
    pub fn new(req: CreatePlantRequest) -> Self {
        let now = Utc::now();
        Plant {
            id: Uuid::new_v4(),
            name: req.name,
            energy_type: req.energy_type,
            capacity_mw: req.capacity_mw,
            location: req.location,
            status: req.status.unwrap_or(PlantStatus::Active),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_update(&mut self, req: UpdatePlantRequest) {
        if let Some(name) = req.name {
            self.name = name;
        }
        if let Some(energy_type) = req.energy_type {
            self.energy_type = energy_type;
        }
        if let Some(capacity_mw) = req.capacity_mw {
            self.capacity_mw = capacity_mw;
        }
        if let Some(location) = req.location {
            self.location = location;
        }
        if let Some(status) = req.status {
            self.status = status;
        }
        self.updated_at = Utc::now();
    }
}
