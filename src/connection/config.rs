use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ConnectionMode {
    #[default]
    Direct,
    Cluster,
    Sentinel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SSHConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub passphrase: Option<String>,
    #[serde(default)]
    pub encrypted_password: Option<EncryptedField>,
    #[serde(default)]
    pub encrypted_passphrase: Option<EncryptedField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EncryptedField {
    pub ciphertext: String,
    pub iv: String,
}

impl EncryptedField {
    pub fn new(ciphertext: String, iv: String) -> Self {
        Self { ciphertext, iv }
    }

    pub fn is_empty(&self) -> bool {
        self.ciphertext.is_empty()
    }
}

impl Default for SSHConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 22,
            username: "root".to_string(),
            password: None,
            private_key_path: None,
            passphrase: None,
            encrypted_password: None,
            encrypted_passphrase: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SSLConfig {
    pub enabled: bool,
    pub ca_cert_path: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

impl Default for SSLConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ca_cert_path: None,
            cert_path: None,
            key_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SentinelConfig {
    pub master_name: String,
    pub nodes: Vec<String>,
    pub password: Option<String>,
}

impl Default for SentinelConfig {
    fn default() -> Self {
        Self {
            master_name: "mymaster".to_string(),
            nodes: vec!["127.0.0.1:26379".to_string()],
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusterConfig {
    pub nodes: Vec<String>,
    #[serde(default)]
    pub read_from_replicas: bool,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            nodes: vec!["127.0.0.1:6379".to_string()],
            read_from_replicas: false,
        }
    }
}

impl ClusterConfig {
    pub fn to_urls(&self) -> Vec<String> {
        self.nodes
            .iter()
            .map(|n| format!("redis://{}", n))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionConfig {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub username: Option<String>,
    pub db: u8,
    #[serde(default)]
    pub connection_timeout: u64,
    #[serde(default)]
    pub mode: ConnectionMode,
    #[serde(default)]
    pub ssh: Option<SSHConfig>,
    #[serde(default)]
    pub ssl: SSLConfig,
    #[serde(default)]
    pub sentinel: Option<SentinelConfig>,
    #[serde(default)]
    pub cluster: Option<ClusterConfig>,
    #[serde(default, skip_serializing)]
    pub use_ssl: bool,
    #[serde(default)]
    pub encrypted_password: Option<EncryptedField>,
    #[serde(default)]
    pub auto_reconnect: bool,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,
    #[serde(default)]
    pub readonly: bool,
}

fn default_heartbeat_interval() -> u64 {
    30
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "New Connection".to_string(),
            host: "127.0.0.1".to_string(),
            port: 6379,
            password: None,
            username: None,
            db: 0,
            connection_timeout: 5000,
            mode: ConnectionMode::Direct,
            ssh: None,
            ssl: SSLConfig::default(),
            sentinel: None,
            cluster: None,
            use_ssl: false,
            encrypted_password: None,
            auto_reconnect: true,
            heartbeat_interval_secs: 30,
            readonly: false,
        }
    }
}

impl ConnectionConfig {
    pub fn to_redis_url(&self) -> String {
        let mut url = String::new();

        url.push_str("redis://");

        if let Some(ref password) = self.password {
            if let Some(ref username) = self.username {
                url.push_str(&format!("{}:{}@", username, password));
            } else {
                url.push_str(&format!(":{}@", password));
            }
        }

        url.push_str(&format!("{}:{}/{}", self.host, self.port, self.db));

        url
    }

    pub fn new(name: impl Into<String>, host: impl Into<String>, port: u16) -> Self {
        Self {
            name: name.into(),
            host: host.into(),
            port,
            ..Default::default()
        }
    }

    pub fn with_ssh(mut self, ssh: SSHConfig) -> Self {
        self.ssh = Some(ssh);
        self
    }

    pub fn with_ssl(mut self, ssl: SSLConfig) -> Self {
        self.ssl = ssl;
        self
    }

    pub fn with_cluster_mode(mut self) -> Self {
        self.mode = ConnectionMode::Cluster;
        self
    }

    pub fn with_cluster(mut self, cluster: ClusterConfig) -> Self {
        self.mode = ConnectionMode::Cluster;
        self.cluster = Some(cluster);
        self
    }

    pub fn with_sentinel(mut self, sentinel: SentinelConfig) -> Self {
        self.mode = ConnectionMode::Sentinel;
        self.sentinel = Some(sentinel);
        self
    }
}
