use proexel_domain::normalize_tag;
use serde::{Deserialize, Serialize};

use crate::migration_support::{default_pending, default_priority};

pub fn normalize_legacy_tag(tag: &str) -> String {
    normalize_tag(tag)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LegacyBundle {
    #[serde(default)]
    pub valves: Vec<LegacyValve>,
    #[serde(default)]
    pub maintenance_records: Vec<LegacyMaintenance>,
    #[serde(default)]
    pub orders: Vec<LegacyOrder>,
    #[serde(default)]
    pub restock_requests: Vec<LegacyRestock>,
    #[serde(default)]
    pub stock: Vec<LegacyStock>,
    #[serde(default)]
    pub suppliers: Vec<LegacySupplier>,
    #[serde(default)]
    pub valve_photos: Vec<LegacyPhoto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyValve {
    pub tag: String,
    #[serde(alias = "zona")]
    pub zone: String,
    #[serde(default, alias = "marca")]
    pub manufacturer: Option<String>,
    #[serde(default, alias = "serie")]
    pub serial: Option<String>,
    #[serde(default, alias = "kit")]
    pub kit_reference: Option<String>,
    #[serde(default, alias = "assento")]
    pub seat: Option<String>,
    #[serde(default)]
    pub dn: Option<String>,
    #[serde(default, alias = "tipo")]
    pub valve_type: Option<String>,
    #[serde(default, alias = "atuador")]
    pub actuator: Option<String>,
    #[serde(default, alias = "fabricacao")]
    pub manufactured_at: Option<String>,
    #[serde(default, alias = "ult_kit")]
    pub last_kit_changed_at: Option<String>,
    #[serde(default, alias = "ult_man")]
    pub last_maintenance_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyMaintenance {
    pub tag: String,
    #[serde(alias = "date")]
    pub performed_at: String,
    pub technician: String,
    #[serde(alias = "type")]
    pub maintenance_type: String,
    #[serde(alias = "service")]
    pub service: String,
    #[serde(default, alias = "kitChanged")]
    pub kit_changed: bool,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, alias = "signature")]
    pub signature_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyOrder {
    #[serde(alias = "zona")]
    pub zone: String,
    #[serde(default, alias = "valveTag", alias = "valve_tag")]
    pub valve_tag: Option<String>,
    #[serde(alias = "observacoes")]
    pub description: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default = "default_pending")]
    pub status: String,
    #[serde(default, alias = "createdBy")]
    pub created_by: Option<String>,
    #[serde(default, alias = "tecnico")]
    pub technician: Option<String>,
    #[serde(default, alias = "data_programada")]
    pub scheduled_for: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyRestock {
    #[serde(alias = "ref", alias = "kit")]
    pub reference: String,
    #[serde(alias = "description")]
    pub reason: String,
    #[serde(default, alias = "created_by", alias = "suggested_by")]
    pub requested_by: Option<String>,
    #[serde(default = "default_pending")]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyStock {
    #[serde(alias = "kit", alias = "ref")]
    pub reference: String,
    #[serde(default)]
    pub quantity: u32,
    #[serde(default, alias = "minQuantity")]
    pub min_quantity: u32,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub brand: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySupplier {
    pub name: String,
    pub contact: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, alias = "created_by")]
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyPhoto {
    pub tag: String,
    #[serde(alias = "storage_path", alias = "photo_url")]
    pub blob_ref: String,
}
