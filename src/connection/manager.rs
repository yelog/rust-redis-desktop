use super::{ConnectionConfig, ConnectionPool, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<Uuid, ConnectionPool>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, config: ConnectionConfig) -> Result<Uuid> {
        let id = config.id;
        let pool = ConnectionPool::new(config).await?;

        let mut connections = self.connections.write().await;
        connections.insert(id, pool);

        Ok(id)
    }

    pub async fn remove_connection(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
    }

    pub async fn get_connection(&self, id: Uuid) -> Option<ConnectionPool> {
        let connections = self.connections.read().await;
        connections.get(&id).cloned()
    }

    pub async fn list_connections(&self) -> Vec<(Uuid, String)> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .map(|(id, pool)| (*id, pool.config().name.clone()))
            .collect()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ConnectionManager {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
        }
    }
}
