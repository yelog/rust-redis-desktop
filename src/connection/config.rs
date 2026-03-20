use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub username: Option<String>,
    pub db: u8,
    pub connection_timeout: u64,
    pub use_ssl: bool,
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
            use_ssl: false,
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
}
