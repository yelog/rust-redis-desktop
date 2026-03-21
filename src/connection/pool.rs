use super::{ConnectionConfig, ConnectionError, Result};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ConnectionPool {
    pub(crate) config: ConnectionConfig,
    pub(crate) connection: Arc<Mutex<Option<ConnectionManager>>>,
}

impl PartialEq for ConnectionPool {
    fn eq(&self, other: &Self) -> bool {
        self.config.id == other.config.id
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connection: Arc::clone(&self.connection),
        }
    }
}

impl ConnectionPool {
    pub async fn new(config: ConnectionConfig) -> Result<Self> {
        let pool = Self {
            config,
            connection: Arc::new(Mutex::new(None)),
        };

        pool.connect().await?;

        Ok(pool)
    }

    async fn connect(&self) -> Result<()> {
        let url = self.config.to_redis_url();

        let client = redis::Client::open(url.as_str())
            .map_err(|e| ConnectionError::InvalidConfig(e.to_string()))?;

        let conn = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.connection_timeout),
            client.get_connection_manager(),
        )
        .await
        .map_err(|_| ConnectionError::Timeout)?
        .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;

        let mut connection = self.connection.lock().await;
        *connection = Some(conn);

        Ok(())
    }

    pub async fn ping(&self) -> Result<String> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.ping()
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn reconnect(&self) -> Result<()> {
        self.connect().await
    }

    pub fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    pub async fn select_database(&self, db: u8) -> Result<()> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            redis::cmd("SELECT")
                .arg(db)
                .query_async::<()>(conn)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
        }

        Ok(())
    }
}
