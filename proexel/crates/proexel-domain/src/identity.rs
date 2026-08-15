use serde::{Deserialize, Serialize};

use crate::ComplexityLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub password_hash: String,
    pub pin_hash: Option<String>,
    pub active: bool,
    #[serde(default)]
    pub maximum_repair_level: ComplexityLevel,
    pub auth_version: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub actor: String,
    pub role: String,
    pub operation: String,
    pub aggregate: String,
    pub aggregate_id: String,
    pub description: Option<String>,
    pub trace_id: Option<String>,
    pub before_json: Option<String>,
    pub after_json: Option<String>,
    pub result: String,
    pub created_at_ms: u64,
}
