use super::TenantPool;
use dashmap::DashMap;
use uuid::Uuid;

pub type Pools = DashMap<Uuid, TenantPool>;

pub struct AppState {
    pub pools: Pools,
}
