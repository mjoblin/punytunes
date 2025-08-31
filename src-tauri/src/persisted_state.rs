use log::error;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{JsonValue, Store, StoreBuilder};

use crate::errors::PunyTunesError;

pub const BACKEND_STORE_FILE: &str = "backend.json";
pub const KEY_LAST_CONNECTED_HOST: &str = "last_connected_host";
pub const KEY_LAST_ACTIVATED_UDN: &str = "last_activated_udn";

pub struct BackendState {
    store: Store<Wry>,
}

impl BackendState {
    pub fn new(app_handle: AppHandle) -> Self {
        let mut store = StoreBuilder::new(app_handle.clone(), BACKEND_STORE_FILE.parse().unwrap()).build();

        match store.load() {
            Ok(_) => {}
            Err(e) => error!("Could not load backend store: {:?}", e),
        }

        BackendState { store }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.store.get(key.to_string())
    }

    pub fn set(&mut self, key: &str, value: JsonValue) -> Result<(), PunyTunesError> {
        match self
            .store
            .insert(key.into(), value)
            .map_err(|e| PunyTunesError::Store(format!("Could not store value for key '{key}': {:?}", e)))
        {
            Ok(result) => match self.store.save() {
                Ok(_) => Ok(result),
                Err(e) => Err(PunyTunesError::Store(format!("Could not save backend store: {:?}", e))),
            },
            Err(e) => Err(e),
        }
    }

    pub fn delete(&mut self, key: &str) -> Result<bool, PunyTunesError> {
        match self.store.delete(key.to_string()) {
            Ok(success) => match self.store.save() {
                Ok(_) => Ok(success),
                Err(e) => Err(PunyTunesError::Store(format!("Could not save backend store: {:?}", e))),
            },
            Err(e) => Err(PunyTunesError::Store(format!("Could not delete backend store key: {:?}", e))),
        }
    }
}
