use std::path::PathBuf;

use appcore_bin::application::run_application;
use proexel_domain::{ComplexityLevel, UserAccount};
use proexel_infrastructure::JsonFileStore;
use serde::Deserialize;

use crate::application::ProexelApplication;

fn state_path() -> PathBuf {
    if let Some(path) = std::env::var_os("PROEXEL_DATA_FILE") {
        return PathBuf::from(path);
    }
    let manifest = std::env::var_os("APPCORE_DEPLOYMENT_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("proexel/apps/service/deployment.local.toml"));
    manifest
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        // The existing path is retained so schema migration can happen in place.
        .join("target/runtime/storage/proexel-state-v1.json")
}

#[derive(Deserialize)]
struct SeedUser {
    id: String,
    email: String,
    name: String,
    role: String,
    password_hash: String,
    #[serde(default)]
    pin_hash: Option<String>,
    #[serde(default)]
    maximum_repair_level: Option<ComplexityLevel>,
    #[serde(default = "default_true")]
    active: bool,
}

fn default_true() -> bool {
    true
}

fn default_repair_level(role: &str) -> ComplexityLevel {
    if matches!(role, "admin" | "chefe") {
        ComplexityLevel::EXPERT
    } else {
        ComplexityLevel::INTERMEDIATE
    }
}

fn seed_users_from_environment(store: &JsonFileStore) -> Result<(), String> {
    let raw = std::env::var("PROEXEL_AUTH_USERS").unwrap_or_else(|_| "[]".to_string());
    let seeds: Vec<SeedUser> =
        serde_json::from_str(&raw).map_err(|_| "auth_users_invalid".to_string())?;
    if seeds.is_empty() {
        return Ok(());
    }
    store.transact(|state| {
        state.seed_users(
            seeds
                .into_iter()
                .map(|seed| {
                    let maximum_repair_level = seed
                        .maximum_repair_level
                        .unwrap_or_else(|| default_repair_level(&seed.role));
                    UserAccount {
                        id: seed.id,
                        email: seed.email,
                        name: seed.name,
                        role: seed.role,
                        password_hash: seed.password_hash,
                        pin_hash: seed.pin_hash,
                        active: seed.active,
                        maximum_repair_level,
                        auth_version: 1,
                        created_at_ms: 0,
                        updated_at_ms: 0,
                    }
                })
                .collect(),
        )
    })
}

pub(crate) fn run() {
    if let Err(error) = crate::release_runtime::configure() {
        eprintln!("proexel release configuration failed: {error}");
        std::process::exit(1);
    }
    let store = match JsonFileStore::new(state_path()) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("proexel storage failed: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = seed_users_from_environment(&store) {
        eprintln!("proexel identity seed failed: {error}");
        std::process::exit(1);
    }
    if let Err(error) = run_application(&ProexelApplication { store }) {
        eprintln!("proexel service failed: {error}");
        std::process::exit(1);
    }
}
