use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config;
use crate::db;
use crate::session::SessionActorHandle;
use axum::extract::FromRef;
use sqlx::SqlitePool;
// use uuid;



// pub type Sessions = Arc<RwLock<HashMap<uuid::Uuid, SessionActorHandle>>>;
pub type Sessions = Arc<RwLock<HashMap<String, SessionActorHandle>>>;
#[derive(Debug, Clone)]
pub struct PSState {
    pub pool: SqlitePool,
    pub active_sessions: Sessions,
}

impl PSState {
    pub async fn init(ps_config: config::Config) -> Result<PSState, Box<dyn std::error::Error>> {
        let pool: sqlx::Pool<sqlx::Sqlite> = db::init(&ps_config.sqlite_config).await?;
        let active_sessions = Arc::new(RwLock::new(HashMap::new()));
        Ok(PSState {
            pool,
            active_sessions,
        })
    }
}

impl FromRef<PSState> for SqlitePool {
    fn from_ref(input: &PSState) -> Self {
        input.pool.clone()
    }
}

impl FromRef<PSState> for Sessions {
    fn from_ref(input: &PSState) -> Self {
        input.active_sessions.clone()
    }
}
