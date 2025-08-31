use std::sync::Mutex;

use crate::persisted_state::BackendState;

pub struct PersistedBackendState(pub Mutex<BackendState>);
