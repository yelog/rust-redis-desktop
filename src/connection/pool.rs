use super::{
    ConnectionConfig, ConnectionError, ConnectionMode, Result, SSHTunnel, SSHTunnelConfig,
};
use redis::aio::ConnectionManager;
use redis::cluster::ClusterClient;
use redis::cluster_async::ClusterConnection;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

const HEARTBEAT_INTERVAL_SECS: u64 = 30;
const MAX_CONSECUTIVE_FAILURES: u32 = 3;
const RECONNECT_DELAY_SECS: u64 = 1;
const DEFAULT_POOL_SIZE: usize = 5;

pub const WRITE_COMMANDS: &[&str] = &[
    "SET", "SETEX", "SETNX", "MSET", "SETRANGE", "APPEND",
    "DEL", "UNLINK",
    "HSET", "HSETNX", "HMSET", "HDEL", "HINCRBY", "HINCRBYFLOAT",
    "LPUSH", "RPUSH", "LSET", "LREM", "LPOP", "RPOP", "LINSERT", "LPUSHX", "RPUSHX",
    "SADD", "SREM", "SMOVE", "SPOP",
    "ZADD", "ZREM", "ZINCRBY", "ZPOPMAX", "ZPOPMIN",
    "XADD", "XDEL", "XTRIM", "XGROUP", "XSETID",
    "INCR", "INCRBY", "DECR", "DECRBY", "INCRBYFLOAT",
    "EXPIRE", "PEXPIRE", "EXPIREAT", "PEXPIREAT", "PERSIST",
    "RENAME", "RENAMENX",
    "FLUSHDB", "FLUSHALL",
    "BITSET", "BITFIELD",
    "PFADD", "PFMERGE",
    "GEOADD",
    "SINTERSTORE", "SUNIONSTORE", "SDIFFSTORE",
    "ZUNIONSTORE", "ZINTERSTORE",
    "SORT",
    "MOVE",
    "MIGRATE",
    "RESTORE",
    "EVAL", "EVALSHA",
    "SCRIPT",
    "PUBLISH",
    "SWAPDB",
    "COPY",
];

pub enum RedisConnection {
    Single(ConnectionManager),
    Cluster(ClusterConnection),
}

impl fmt::Debug for RedisConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedisConnection::Single(_) => write!(f, "RedisConnection::Single"),
            RedisConnection::Cluster(_) => write!(f, "RedisConnection::Cluster"),
        }
    }
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

    pub async fn get_string(&mut self, key: &str) -> Result<String> {
        let mut cmd = redis::cmd("GET");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn get_bytes(&mut self, key: &str) -> Result<Vec<u8>> {
        let mut cmd = redis::cmd("GET");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn set_string(&mut self, key: &str, value: &str) -> Result<()> {
        let mut cmd = redis::cmd("SET");
        cmd.arg(key).arg(value);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn del_key(&mut self, key: &str) -> Result<i32> {
        let mut cmd = redis::cmd("DEL");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn ttl(&mut self, key: &str) -> Result<i64> {
        let mut cmd = redis::cmd("TTL");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn expire(&mut self, key: &str, seconds: i64) -> Result<i32> {
        let mut cmd = redis::cmd("EXPIRE");
        cmd.arg(key).arg(seconds);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn persist(&mut self, key: &str) -> Result<i32> {
        let mut cmd = redis::cmd("PERSIST");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn rename(&mut self, old_key: &str, new_key: &str) -> Result<()> {
        let mut cmd = redis::cmd("RENAME");
        cmd.arg(old_key).arg(new_key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn hgetall(
        &mut self,
        key: &str,
    ) -> Result<std::collections::HashMap<String, String>> {
        let mut cmd = redis::cmd("HGETALL");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn hset(&mut self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut cmd = redis::cmd("HSET");
        cmd.arg(key).arg(field).arg(value);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn hdel(&mut self, key: &str, field: &str) -> Result<i32> {
        let mut cmd = redis::cmd("HDEL");
        cmd.arg(key).arg(field);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn hlen(&mut self, key: &str) -> Result<u64> {
        let mut cmd = redis::cmd("HLEN");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn lrange(&mut self, key: &str, start: isize, stop: isize) -> Result<Vec<String>> {
        let mut cmd = redis::cmd("LRANGE");
        cmd.arg(key).arg(start).arg(stop);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn lpush(&mut self, key: &str, value: &str) -> Result<i64> {
        let mut cmd = redis::cmd("LPUSH");
        cmd.arg(key).arg(value);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn rpush(&mut self, key: &str, value: &str) -> Result<i64> {
        let mut cmd = redis::cmd("RPUSH");
        cmd.arg(key).arg(value);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn lpop(&mut self, key: &str) -> Result<Option<String>> {
        let mut cmd = redis::cmd("LPOP");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn rpop(&mut self, key: &str) -> Result<Option<String>> {
        let mut cmd = redis::cmd("RPOP");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn lset(&mut self, key: &str, index: isize, value: &str) -> Result<()> {
        let mut cmd = redis::cmd("LSET");
        cmd.arg(key).arg(index).arg(value);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn lrem(&mut self, key: &str, count: isize, value: &str) -> Result<i64> {
        let mut cmd = redis::cmd("LREM");
        cmd.arg(key).arg(count).arg(value);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn llen(&mut self, key: &str) -> Result<u64> {
        let mut cmd = redis::cmd("LLEN");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn smembers(&mut self, key: &str) -> Result<Vec<String>> {
        let mut cmd = redis::cmd("SMEMBERS");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn sadd(&mut self, key: &str, member: &str) -> Result<i32> {
        let mut cmd = redis::cmd("SADD");
        cmd.arg(key).arg(member);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn srem(&mut self, key: &str, member: &str) -> Result<i32> {
        let mut cmd = redis::cmd("SREM");
        cmd.arg(key).arg(member);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn scard(&mut self, key: &str) -> Result<u64> {
        let mut cmd = redis::cmd("SCARD");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn zrange_withscores(
        &mut self,
        key: &str,
        start: isize,
        stop: isize,
    ) -> Result<Vec<(String, f64)>> {
        let mut cmd = redis::cmd("ZRANGE");
        cmd.arg(key).arg(start).arg(stop).arg("WITHSCORES");
        self.execute_cmd(&mut cmd).await
    }

    pub async fn zadd(&mut self, key: &str, score: f64, member: &str) -> Result<i32> {
        let mut cmd = redis::cmd("ZADD");
        cmd.arg(key).arg(score).arg(member);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn zrem(&mut self, key: &str, member: &str) -> Result<i32> {
        let mut cmd = redis::cmd("ZREM");
        cmd.arg(key).arg(member);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn zcard(&mut self, key: &str) -> Result<u64> {
        let mut cmd = redis::cmd("ZCARD");
        cmd.arg(key);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn rpush_vec(&mut self, key: &str, values: Vec<String>) -> Result<()> {
        let mut cmd = redis::cmd("RPUSH");
        cmd.arg(key);
        for v in &values {
            cmd.arg(v);
        }
        self.execute_cmd(&mut cmd).await
    }

    pub async fn sadd_vec(&mut self, key: &str, members: Vec<String>) -> Result<()> {
        let mut cmd = redis::cmd("SADD");
        cmd.arg(key);
        for m in &members {
            cmd.arg(m);
        }
        self.execute_cmd(&mut cmd).await
    }

    pub async fn zadd_str(&mut self, key: &str, score: f64, member: &str) -> Result<i32> {
        let mut cmd = redis::cmd("ZADD");
        cmd.arg(key).arg(score).arg(member);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn lpush_str(&mut self, key: &str, value: &str) -> Result<i64> {
        let mut cmd = redis::cmd("LPUSH");
        cmd.arg(key).arg(value);
        self.execute_cmd(&mut cmd).await
    }

    pub async fn rpush_str(&mut self, key: &str, value: &str) -> Result<i64> {
        let mut cmd = redis::cmd("RPUSH");
        cmd.arg(key).arg(value);
        self.execute_cmd(&mut cmd).await
    }
}

pub struct ConnectionPool {
    pub(crate) config: ConnectionConfig,
    pub(crate) connection: Arc<Mutex<Option<RedisConnection>>>,
    pub(crate) selected_db: Arc<AtomicU8>,
    ssh_tunnel: Arc<Mutex<Option<SSHTunnel>>>,
    pool_semaphore: Arc<Semaphore>,
    pool_size: usize,
}

impl fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionPool")
            .field("config", &self.config)
            .field("selected_db", &self.selected_db)
            .field("pool_size", &self.pool_size)
            .finish()
    }
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
            ssh_tunnel: Arc::clone(&self.ssh_tunnel),
            pool_semaphore: Arc::clone(&self.pool_semaphore),
            pool_size: self.pool_size,
        }
    }
}

impl ConnectionPool {
    pub async fn new(config: ConnectionConfig) -> Result<Self> {
        Self::with_pool_size(config, DEFAULT_POOL_SIZE).await
    }

    pub async fn with_pool_size(config: ConnectionConfig, pool_size: usize) -> Result<Self> {
        let pool = Self {
            selected_db: Arc::new(AtomicU8::new(config.db)),
            config,
            connection: Arc::new(Mutex::new(None)),
            ssh_tunnel: Arc::new(Mutex::new(None)),
            pool_semaphore: Arc::new(Semaphore::new(pool_size)),
            pool_size,
        };

        pool.connect().await?;

        Ok(pool)
    }

    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    pub async fn available_connections(&self) -> usize {
        self.pool_semaphore.available_permits()
    }

    async fn connect(&self) -> Result<()> {
        match self.config.mode {
            ConnectionMode::Direct => self.connect_single().await,
            ConnectionMode::Cluster => self.connect_cluster().await,
            ConnectionMode::Sentinel => self.connect_single().await,
        }
    }

    async fn setup_ssh_tunnel(&self) -> Result<u16> {
        let ssh_config = self.config.ssh.as_ref().ok_or_else(|| {
            ConnectionError::InvalidConfig("SSH configuration is required".to_string())
        })?;

        let tunnel_config = SSHTunnelConfig {
            ssh_host: ssh_config.host.clone(),
            ssh_port: ssh_config.port,
            ssh_username: ssh_config.username.clone(),
            ssh_password: ssh_config.password.clone(),
            ssh_private_key_path: ssh_config.private_key_path.clone(),
            ssh_passphrase: ssh_config.passphrase.clone(),
            remote_host: self.config.host.clone(),
            remote_port: self.config.port,
        };

        let tunnel =
            SSHTunnel::start(tunnel_config).map_err(|e| ConnectionError::ConnectionFailed(e))?;

        let local_port = tunnel.local_port();

        let mut ssh_tunnel = self.ssh_tunnel.lock().await;
        *ssh_tunnel = Some(tunnel);

        Ok(local_port)
    }

    async fn connect_single(&self) -> Result<()> {
        let (host, port) = if self.config.ssh.is_some() {
            let local_port = self.setup_ssh_tunnel().await?;
            ("127.0.0.1".to_string(), local_port)
        } else {
            (self.config.host.clone(), self.config.port)
        };

        let scheme = if self.config.ssl.enabled {
            "rediss"
        } else {
            "redis"
        };
        let mut url = format!("{}://", scheme);

        if let Some(ref password) = self.config.password {
            if let Some(ref username) = self.config.username {
                url.push_str(&format!("{}:{}@", username, password));
            } else {
                url.push_str(&format!(":{}@", password));
            }
        }

        url.push_str(&format!("{}:{}/{}", host, port, self.config.db));

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

        let scheme = if self.config.ssl.enabled {
            "rediss"
        } else {
            "redis"
        };

        let nodes = if cluster_config.nodes.is_empty() {
            vec![format!(
                "{}://{}:{}",
                scheme, self.config.host, self.config.port
            )]
        } else {
            cluster_config
                .nodes
                .iter()
                .map(|n| format!("{}://{}", scheme, n))
                .collect()
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

    pub fn is_readonly(&self) -> bool {
        self.config.readonly
    }

    pub fn check_write_permission(&self, command: &str) -> Result<()> {
        if !self.config.readonly {
            return Ok(());
        }

        let cmd_upper = command.to_uppercase();
        if WRITE_COMMANDS.contains(&cmd_upper.as_str()) {
            return Err(ConnectionError::ReadonlyMode);
        }

        Ok(())
    }

    pub fn check_raw_command_permission(&self, command: &str) -> Result<()> {
        if !self.config.readonly {
            return Ok(());
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let cmd = parts[0].to_uppercase();
        
        if WRITE_COMMANDS.contains(&cmd.as_str()) {
            return Err(ConnectionError::ReadonlyMode);
        }

        if cmd == "COMMAND" || cmd == "INFO" || cmd == "PING" {
            return Ok(());
        }

        Ok(())
    }

    pub async fn ping(&self) -> Result<String> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("PING")).await
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
            conn.execute_cmd::<()>(&mut redis::cmd("SELECT").arg(db))
                .await?;
        }

        Ok(())
    }

    pub async fn check_connection_health(&self) -> bool {
        match self.ping().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub async fn ensure_connection(&self) -> Result<()> {
        if self.check_connection_health().await {
            return Ok(());
        }

        tracing::warn!("Connection health check failed, attempting reconnect");
        self.reconnect().await
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_check: Option<Instant>,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            is_healthy: true,
            last_check: None,
            consecutive_failures: 0,
            last_error: None,
        }
    }
}

pub struct ConnectionHealthMonitor {
    pool: ConnectionPool,
    status: Arc<Mutex<HealthStatus>>,
    running: Arc<AtomicBool>,
}

impl ConnectionHealthMonitor {
    pub fn new(pool: ConnectionPool) -> Self {
        Self {
            pool,
            status: Arc::new(Mutex::new(HealthStatus::default())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let pool = self.pool.clone();
        let status = self.status.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

            while running.load(Ordering::Relaxed) {
                interval.tick().await;

                let is_healthy = pool.check_connection_health().await;

                let mut status_guard = status.lock().await;
                status_guard.last_check = Some(Instant::now());

                if is_healthy {
                    status_guard.is_healthy = true;
                    status_guard.consecutive_failures = 0;
                    status_guard.last_error = None;
                    tracing::debug!("Connection health check passed");
                } else {
                    status_guard.consecutive_failures += 1;
                    status_guard.is_healthy = false;

                    if status_guard.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        tracing::warn!(
                            "Connection failed {} times, attempting reconnect",
                            status_guard.consecutive_failures
                        );

                        drop(status_guard);

                        if let Err(e) = pool.reconnect().await {
                            let mut status_guard = status.lock().await;
                            status_guard.last_error = Some(e.to_string());
                            tracing::error!("Reconnect failed: {}", e);

                            tokio::time::sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
                        } else {
                            let mut status_guard = status.lock().await;
                            status_guard.is_healthy = true;
                            status_guard.consecutive_failures = 0;
                            status_guard.last_error = None;
                            tracing::info!("Reconnect successful");
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub async fn get_status(&self) -> HealthStatus {
        self.status.lock().await.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
