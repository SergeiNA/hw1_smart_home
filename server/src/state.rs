use smart_home::smart_home::SmartHome;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type AppState = Arc<RwLock<SmartHome>>;

pub fn spawn_app_state() -> AppState {
    Arc::new(RwLock::new(SmartHome::default()))
}
