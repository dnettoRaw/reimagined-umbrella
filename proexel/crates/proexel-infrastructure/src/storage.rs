use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use proexel_application::ApplicationState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    pub const INITIAL: Self = Self(1);
}

#[derive(Clone)]
pub struct JsonFileStore {
    inner: Arc<Mutex<StoreInner>>,
}

struct StoreInner {
    path: PathBuf,
    state: ApplicationState,
}

impl JsonFileStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let state = load_state(&path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(StoreInner { path, state })),
        })
    }

    pub fn set_path(&self, path: impl Into<PathBuf>) -> Result<(), String> {
        let path = path.into();
        let state = load_state(&path)?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| "storage_lock_poisoned".to_string())?;
        inner.path = path;
        inner.state = state;
        Ok(())
    }

    pub fn read(&self) -> Result<ApplicationState, String> {
        self.inner
            .lock()
            .map(|inner| inner.state.clone())
            .map_err(|_| "storage_lock_poisoned".to_string())
    }

    pub fn transact<T>(
        &self,
        operation: impl FnOnce(&mut ApplicationState) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| "storage_lock_poisoned".to_string())?;
        let mut candidate = inner.state.clone();
        let result = operation(&mut candidate)?;
        persist_state(&inner.path, &candidate)?;
        inner.state = candidate;
        Ok(result)
    }
}

fn load_state(path: &Path) -> Result<ApplicationState, String> {
    if !path.exists() {
        return Ok(ApplicationState::default());
    }
    let bytes = fs::read(path).map_err(|error| format!("storage_read_failed: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("storage_decode_failed: {error}"))
}

fn persist_state(path: &Path, state: &ApplicationState) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "storage_parent_missing".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("storage_create_dir_failed: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|error| format!("storage_encode_failed: {error}"))?;
    let mut file =
        File::create(&temporary).map_err(|error| format!("storage_create_failed: {error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("storage_write_failed: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("storage_sync_failed: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("storage_commit_failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_is_durable_and_failed_write_is_not_committed() {
        let path = std::env::temp_dir().join(format!("proexel-store-{}.json", std::process::id()));
        let store = JsonFileStore::new(&path).unwrap();
        store
            .transact(|state| {
                state.schema_version = 7;
                Ok(())
            })
            .unwrap();
        assert_eq!(
            JsonFileStore::new(&path)
                .unwrap()
                .read()
                .unwrap()
                .schema_version,
            7
        );
        let result: Result<(), String> = store.transact(|state| {
            state.schema_version = 9;
            Err("stop".to_string())
        });
        assert_eq!(result, Err("stop".to_string()));
        assert_eq!(store.read().unwrap().schema_version, 7);
        let _ = fs::remove_file(path);
    }
}
