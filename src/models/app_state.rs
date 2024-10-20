use crate::models::TenantPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct AppState {
    pub pools: Arc<Mutex<HashMap<Uuid, Arc<Mutex<TenantPool>>>>>,
}
