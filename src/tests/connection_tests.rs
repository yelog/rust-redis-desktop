use crate::config::ConfigStorage;
use crate::connection::{
    ClusterConfig, ConnectionConfig, ConnectionMode, SSHConfig, SSLConfig, SentinelConfig,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_connection_config() {
        let config = ConnectionConfig::new("test-direct", "localhost", 6379);

        assert_eq!(config.name, "test-direct");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 6379);
        assert_eq!(config.mode, ConnectionMode::Direct);
        assert_eq!(config.db, 0);
        assert!(!config.readonly);
    }

    #[test]
    fn test_connection_config_with_password() {
        let config = ConnectionConfig {
            password: Some("secret123".to_string()),
            ..ConnectionConfig::new("test-auth", "localhost", 6379)
        };

        assert_eq!(config.password, Some("secret123".to_string()));
        let url = config.to_redis_url();
        assert!(url.contains(":secret123@"));
    }

    #[test]
    fn test_connection_config_with_username() {
        let config = ConnectionConfig {
            username: Some("admin".to_string()),
            password: Some("secret123".to_string()),
            ..ConnectionConfig::new("test-auth", "localhost", 6379)
        };

        let url = config.to_redis_url();
        assert!(url.contains("admin:secret123@"));
    }

    #[test]
    fn test_cluster_mode_config() {
        let cluster_config = ClusterConfig {
            nodes: vec![
                "localhost:7000".to_string(),
                "localhost:7001".to_string(),
                "localhost:7002".to_string(),
            ],
            read_from_replicas: true,
        };

        let config = ConnectionConfig::new("test-cluster", "localhost", 7000)
            .with_cluster(cluster_config.clone());

        assert_eq!(config.mode, ConnectionMode::Cluster);
        assert!(config.cluster.is_some());
        let cluster = config.cluster.unwrap();
        assert_eq!(cluster.nodes.len(), 3);
        assert!(cluster.read_from_replicas);
    }

    #[test]
    fn test_sentinel_mode_config() {
        let sentinel_config = SentinelConfig {
            master_name: "mymaster".to_string(),
            nodes: vec!["localhost:26379".to_string()],
            password: None,
        };

        let config = ConnectionConfig::new("test-sentinel", "localhost", 26379)
            .with_sentinel(sentinel_config.clone());

        assert_eq!(config.mode, ConnectionMode::Sentinel);
        assert!(config.sentinel.is_some());
        let sentinel = config.sentinel.unwrap();
        assert_eq!(sentinel.master_name, "mymaster");
        assert_eq!(sentinel.nodes.len(), 1);
    }

    #[test]
    fn test_ssh_tunnel_config() {
        let ssh_config = SSHConfig {
            host: "ssh.example.com".to_string(),
            port: 22,
            username: "testuser".to_string(),
            password: Some("sshpass".to_string()),
            private_key_path: None,
            passphrase: None,
            encrypted_password: None,
            encrypted_passphrase: None,
        };

        let config =
            ConnectionConfig::new("test-ssh", "127.0.0.1", 6379).with_ssh(ssh_config.clone());

        assert!(config.ssh.is_some());
        let ssh = config.ssh.unwrap();
        assert_eq!(ssh.host, "ssh.example.com");
        assert_eq!(ssh.port, 22);
        assert_eq!(ssh.username, "testuser");
    }

    #[test]
    fn test_ssh_with_private_key() {
        let ssh_config = SSHConfig {
            host: "ssh.example.com".to_string(),
            port: 22,
            username: "testuser".to_string(),
            password: None,
            private_key_path: Some("/home/user/.ssh/id_rsa".to_string()),
            passphrase: Some("keypass".to_string()),
            encrypted_password: None,
            encrypted_passphrase: None,
        };

        let config = ConnectionConfig::new("test-ssh-key", "127.0.0.1", 6379).with_ssh(ssh_config);

        assert!(config.ssh.is_some());
        assert!(config.ssh.as_ref().unwrap().private_key_path.is_some());
    }

    #[test]
    fn test_ssl_config() {
        let ssl_config = SSLConfig {
            enabled: true,
            ca_cert_path: Some("/path/to/ca.crt".to_string()),
            cert_path: Some("/path/to/client.crt".to_string()),
            key_path: Some("/path/to/client.key".to_string()),
        };

        let config =
            ConnectionConfig::new("test-ssl", "localhost", 6382).with_ssl(ssl_config.clone());

        assert!(config.ssl.enabled);
        assert!(config.ssl.ca_cert_path.is_some());
    }

    #[test]
    fn test_readonly_mode() {
        let config = ConnectionConfig {
            readonly: true,
            ..ConnectionConfig::new("test-readonly", "localhost", 6379)
        };

        assert!(config.readonly);
    }

    #[test]
    fn test_connection_timeout() {
        let config = ConnectionConfig {
            connection_timeout: 10000,
            ..ConnectionConfig::new("test-timeout", "localhost", 6379)
        };

        assert_eq!(config.connection_timeout, 10000);
    }

    #[test]
    fn test_auto_reconnect() {
        let config = ConnectionConfig {
            auto_reconnect: false,
            ..ConnectionConfig::new("test-reconnect", "localhost", 6379)
        };

        assert!(!config.auto_reconnect);
    }

    #[test]
    fn test_heartbeat_interval() {
        let config = ConnectionConfig {
            heartbeat_interval_secs: 60,
            ..ConnectionConfig::new("test-heartbeat", "localhost", 6379)
        };

        assert_eq!(config.heartbeat_interval_secs, 60);
    }

    #[test]
    fn test_database_selection() {
        let config = ConnectionConfig {
            db: 5,
            ..ConnectionConfig::new("test-db", "localhost", 6379)
        };

        assert_eq!(config.db, 5);
        let url = config.to_redis_url();
        assert!(url.ends_with("/5"));
    }

    #[test]
    fn test_cluster_config_urls() {
        let cluster = ClusterConfig {
            nodes: vec![
                "192.168.1.1:7000".to_string(),
                "192.168.1.2:7001".to_string(),
            ],
            read_from_replicas: false,
        };

        let urls = cluster.to_urls();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "redis://192.168.1.1:7000");
        assert_eq!(urls[1], "redis://192.168.1.2:7001");
    }

    #[test]
    fn test_default_connection_config() {
        let config = ConnectionConfig::default();

        assert_eq!(config.mode, ConnectionMode::Direct);
        assert!(!config.ssl.enabled);
        assert!(config.ssh.is_none());
        assert!(config.cluster.is_none());
        assert!(config.sentinel.is_none());
        assert!(!config.readonly);
        assert!(config.auto_reconnect);
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.connection_timeout, 5000);
    }

    #[test]
    fn test_connection_mode_equality() {
        assert_eq!(ConnectionMode::Direct, ConnectionMode::Direct);
        assert_eq!(ConnectionMode::Cluster, ConnectionMode::Cluster);
        assert_eq!(ConnectionMode::Sentinel, ConnectionMode::Sentinel);
        assert_ne!(ConnectionMode::Direct, ConnectionMode::Cluster);
    }

    #[test]
    fn test_config_serialization() {
        let config = ConnectionConfig::new("test-serialize", "localhost", 6379).with_cluster(
            ClusterConfig {
                nodes: vec!["localhost:7000".to_string()],
                read_from_replicas: false,
            },
        );

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"mode\":\"Cluster\""));
        assert!(json.contains("\"test-serialize\""));

        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-serialize");
        assert_eq!(deserialized.mode, ConnectionMode::Cluster);
    }

    #[test]
    fn test_config_with_all_options() {
        let config = ConnectionConfig {
            id: uuid::Uuid::nil(),
            name: "full-config".to_string(),
            host: "redis.example.com".to_string(),
            port: 6380,
            password: Some("password".to_string()),
            username: Some("user".to_string()),
            db: 3,
            connection_timeout: 15000,
            mode: ConnectionMode::Cluster,
            ssh: Some(SSHConfig::default()),
            ssl: SSLConfig {
                enabled: true,
                ca_cert_path: Some("/certs/ca.crt".to_string()),
                cert_path: None,
                key_path: None,
            },
            sentinel: None,
            cluster: Some(ClusterConfig::default()),
            use_ssl: true,
            encrypted_password: None,
            auto_reconnect: true,
            heartbeat_interval_secs: 60,
            readonly: true,
            order: 5,
        };

        assert!(config.ssh.is_some());
        assert!(config.ssl.enabled);
        assert!(config.cluster.is_some());
        assert!(config.readonly);
        assert_eq!(config.order, 5);
    }

    #[test]
    fn test_encrypt_connection_credentials_clears_plaintext() {
        let config = ConnectionConfig {
            password: Some("secret123".to_string()),
            ..ConnectionConfig::new("test-auth", "localhost", 6379)
        };

        let encrypted = config.encrypt_credentials().unwrap();

        assert_eq!(encrypted.password, None);
        assert!(encrypted.encrypted_password.is_some());
        assert!(!encrypted.encrypted_password.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_decrypt_connection_credentials_restores_plaintext() {
        let config = ConnectionConfig {
            password: Some("secret123".to_string()),
            ..ConnectionConfig::new("test-auth", "localhost", 6379)
        };

        let encrypted = config.encrypt_credentials().unwrap();
        let decrypted = encrypted.decrypt_credentials().unwrap();

        assert_eq!(decrypted.password, Some("secret123".to_string()));
    }

    #[test]
    fn test_encrypt_ssh_credentials_clears_plaintext() {
        let ssh_config = SSHConfig {
            host: "ssh.example.com".to_string(),
            port: 22,
            username: "testuser".to_string(),
            password: Some("sshpass".to_string()),
            private_key_path: None,
            passphrase: Some("keypass".to_string()),
            encrypted_password: None,
            encrypted_passphrase: None,
        };

        let config = ConnectionConfig::new("test-ssh", "127.0.0.1", 6379).with_ssh(ssh_config);
        let encrypted = config.encrypt_credentials().unwrap();
        let ssh = encrypted.ssh.unwrap();

        assert_eq!(ssh.password, None);
        assert_eq!(ssh.passphrase, None);
        assert!(ssh.encrypted_password.is_some());
        assert!(ssh.encrypted_passphrase.is_some());
    }

    #[test]
    fn test_decrypt_ssh_credentials_restores_plaintext() {
        let ssh_config = SSHConfig {
            host: "ssh.example.com".to_string(),
            port: 22,
            username: "testuser".to_string(),
            password: Some("sshpass".to_string()),
            private_key_path: None,
            passphrase: Some("keypass".to_string()),
            encrypted_password: None,
            encrypted_passphrase: None,
        };

        let config = ConnectionConfig::new("test-ssh", "127.0.0.1", 6379).with_ssh(ssh_config);
        let encrypted = config.encrypt_credentials().unwrap();
        let decrypted = encrypted.decrypt_credentials().unwrap();
        let ssh = decrypted.ssh.unwrap();

        assert_eq!(ssh.password, Some("sshpass".to_string()));
        assert_eq!(ssh.passphrase, Some("keypass".to_string()));
    }

    #[test]
    fn test_config_storage_round_trip_encrypts_on_disk() {
        let temp_dir = std::env::temp_dir().join("rust-redis-desktop-test");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let storage = ConfigStorage::new_temp().unwrap();
        let config = ConnectionConfig {
            password: Some("secret123".to_string()),
            ..ConnectionConfig::new("test-storage", "localhost", 6379)
        };

        storage.save_connection(config).unwrap();

        let raw = std::fs::read_to_string(temp_dir.join("config.json")).unwrap();
        assert!(!raw.contains("secret123"));
        assert!(raw.contains("encrypted_password"));

        let loaded = storage.load_connections().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].password.as_deref(), Some("secret123"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
