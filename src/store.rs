use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreatePlantRequest, Plant, UpdatePlantRequest};

#[derive(Debug, Clone, Default)]
pub struct PlantStore {
    inner: Arc<RwLock<HashMap<Uuid, Plant>>>,
}

impl PlantStore {
    pub fn new() -> Self {
        PlantStore {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn list(&self) -> Result<Vec<Plant>, AppError> {
        let store = self.inner.read().map_err(|_| AppError::LockError)?;
        let mut plants: Vec<Plant> = store.values().cloned().collect();
        plants.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(plants)
    }

    pub fn get(&self, id: Uuid) -> Result<Plant, AppError> {
        let store = self.inner.read().map_err(|_| AppError::LockError)?;
        store
            .get(&id)
            .cloned()
            .ok_or(AppError::NotFound(id.to_string()))
    }

    pub fn create(&self, req: CreatePlantRequest) -> Result<Plant, AppError> {
        let plant = Plant::new(req);
        let mut store = self.inner.write().map_err(|_| AppError::LockError)?;
        store.insert(plant.id, plant.clone());
        Ok(plant)
    }

    pub fn update(&self, id: Uuid, req: UpdatePlantRequest) -> Result<Plant, AppError> {
        let mut store = self.inner.write().map_err(|_| AppError::LockError)?;
        let plant = store.get_mut(&id).ok_or(AppError::NotFound(id.to_string()))?;
        plant.apply_update(req);
        Ok(plant.clone())
    }

    pub fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let mut store = self.inner.write().map_err(|_| AppError::LockError)?;
        store.remove(&id).ok_or(AppError::NotFound(id.to_string()))?;
        Ok(())
    }
}
