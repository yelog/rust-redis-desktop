use super::{ConnectionConfig, ConnectionError, ConnectionMode, Result};
use redis::aio::ConnectionManager;
use redis::cluster::ClusterClient;
use redis::cluster_async::ClusterConnection;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum RedisConnection {
    Single(ConnectionManager),
    Cluster(ClusterConnection),
}

impl RedisConnection {
    pub async fn execute_cmd<V>(&mut self, cmd: &mut redis::Cmd) -> Result<V>
    where
        V: redis::FromRedisValue,
    {
        match self {
            RedisConnection::Single(c) => cmd
                .query_async(c)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string())),
            RedisConnection::Cluster(c) => cmd
                .query_async(c)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct ConnectionPool {
    pub(crate) config: ConnectionConfig,
    pub(crate) connection: Arc<Mutex<Option<RedisConnection>>>,
    pub(crate) selected_db: Arc<AtomicU8>,
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
            selected_db: Arc::clone(&self.selected_db),
        }
    }
}

impl ConnectionPool {
    pub async fn new(config: ConnectionConfig) -> Result<Self> {
        let pool = Self {
            selected_db: Arc::new(AtomicU8::new(config.db)),
            config,
            connection: Arc::new(Mutex::new(None)),
        };

        pool.connect().await?;

        Ok(pool)
    }

    async fn connect(&self) -> Result<()> {
        match self.config.mode {
            ConnectionMode::Direct => self.connect_single().await,
            ConnectionMode::Cluster => self.connect_cluster().await,
            ConnectionMode::Sentinel => self.connect_single().await,
        }
    }

    async fn connect_single(&self) -> Result<()> {
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

        let mut conn = conn;
        self.ensure_selected_database(&mut conn).await?;

        let mut connection = self.connection.lock().await;
        *connection = Some(RedisConnection::Single(conn));

        Ok(())
    }

    async fn connect_cluster(&self) -> Result<()> {
        let cluster_config = self.config.cluster.as_ref().ok_or_else(|| {
            ConnectionError::InvalidConfig("Cluster configuration is required".to_string())
        })?;

        let nodes = if cluster_config.nodes.is_empty() {
            vec![format!("redis://{}:{}", self.config.host, self.config.port)]
        } else {
            cluster_config.to_urls()
        };

        let mut builder = ClusterClient::builder(nodes);

        if let Some(ref password) = self.config.password {
            builder = builder.password(password.clone());
        }

        if let Some(ref username) = self.config.username {
            builder = builder.username(username.clone());
        }

        let client = builder
            .build()
            .map_err(|e| ConnectionError::InvalidConfig(e.to_string()))?;

        let conn = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.connection_timeout),
            client.get_async_connection(),
        )
        .await
        .map_err(|_| ConnectionError::Timeout)?
        .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;

        let mut connection = self.connection.lock().await;
        *connection = Some(RedisConnection::Cluster(conn));

        Ok(())
    }

    pub async fn ping(&self) -> Result<String> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(redis::cmd("PING")).await
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

    pub fn current_db(&self) -> u8 {
        self.selected_db.load(Ordering::Relaxed)
    }

    pub(crate) async fn ensure_selected_database(
        &self,
        conn: &mut ConnectionManager,
    ) -> Result<()> {
        let db = self.current_db();
        redis::cmd("SELECT")
            .arg(db)
            .query_async::<()>(conn)
            .await
            .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
    }

    pub async fn select_database(&self, db: u8) -> Result<()> {
        self.selected_db.store(db, Ordering::Relaxed);
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            match conn {
                RedisConnection::Single(c) => {
                    conn.execute_cmd::<()>(redis::cmd("SELECT").arg(db))
                        .await?;
                }
                RedisConnection::Cluster(_) => {}
            }
        }

        Ok(())
    }
}