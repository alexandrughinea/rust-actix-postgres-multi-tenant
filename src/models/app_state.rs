use crate::models::TenantPool;
use std::collections::HashMap;
use tokio::sync::{Mutex};
use std::sync::{Arc};
use uuid::Uuid;

pub struct AppState {
    pub pools: Arc<Mutex<HashMap<Uuid, Arc<Mutex<TenantPool>>>>>,
}
