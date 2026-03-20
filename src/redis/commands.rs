use super::{KeyInfo, KeyType};
use crate::connection::{ConnectionError, ConnectionPool, Result};
use crate::ui::add_key_dialog::{HashField, ListValue, SetValue, ZSetMember, StreamEntry};
use redis::AsyncCommands;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ServerInfo {
    pub redis_version: Option<String>,
    pub redis_mode: Option<String>,
    pub os: Option<String>,
    pub arch_bits: Option<String>,
    pub process_id: Option<u32>,
    pub tcp_port: Option<u16>,
    pub uptime_in_seconds: Option<u64>,
    pub uptime_in_days: Option<u64>,
    pub connected_clients: Option<u64>,
    pub max_clients: Option<u64>,
    pub total_connections_received: Option<u64>,
    pub total_commands_processed: Option<u64>,
    pub instantaneous_ops_per_sec: Option<u64>,
    pub total_net_input_bytes: Option<u64>,
    pub total_net_output_bytes: Option<u64>,
    pub used_memory: Option<u64>,
    pub used_memory_human: Option<String>,
    pub used_memory_peak: Option<u64>,
    pub used_memory_peak_human: Option<String>,
    pub used_memory_rss: Option<u64>,
    pub mem_fragmentation_ratio: Option<f64>,
    pub mem_allocator: Option<String>,
    pub rdb_last_save_time: Option<u64>,
    pub rdb_changes_since_last_save: Option<u64>,
    pub aof_enabled: Option<u8>,
    pub aof_rewrite_in_progress: Option<u8>,
    pub keyspace: HashMap<String, u64>,
    pub keys_total: u64,
    pub expires_total: u64,
}

impl ConnectionPool {
    pub async fn scan_keys(&self, pattern: &str, count: usize) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let mut keys = Vec::new();
            let mut cursor: u64 = 0;
            
            loop {
                let result: (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
                    .arg("COUNT")
                    .arg(count)
                    .query_async(conn)
                    .await
                    .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
                
                cursor = result.0;
                keys.extend(result.1);
                
                if cursor == 0 {
                    break;
                }
            }
            
            Ok(keys)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn scan_keys_with_progress<F>(&self, pattern: &str, batch_size: usize, mut on_batch: F) -> Result<usize>
    where
        F: FnMut(usize) + Send,
    {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let mut total_count = 0;
            let mut cursor: u64 = 0;
            
            loop {
                let result: (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
                    .arg("COUNT")
                    .arg(batch_size)
                    .query_async(conn)
                    .await
                    .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
                
                cursor = result.0;
                let batch_len = result.1.len();
                total_count += batch_len;
                
                on_batch(total_count);
                
                if cursor == 0 {
                    break;
                }
            }
            
            Ok(total_count)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_key_type(&self, key: &str) -> Result<KeyType> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let type_str: String = redis::cmd("TYPE")
                .arg(key)
                .query_async(conn)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(KeyType::from(type_str))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_key_info(&self, key: &str) -> Result<KeyInfo> {
        let key_type = self.get_key_type(key).await?;
        
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let ttl: i64 = conn
                .ttl(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            let ttl = if ttl == -1 { None } else { Some(ttl) };
            
            Ok(KeyInfo {
                name: key.to_string(),
                key_type,
                ttl,
                size: None,
            })
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_string_value(&self, key: &str) -> Result<String> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.get(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_string_bytes(&self, key: &str) -> Result<Vec<u8>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.get(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn set_string_value(&self, key: &str, value: &str) -> Result<()> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.set(key, value)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn delete_key(&self, key: &str) -> Result<bool> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let deleted: i32 = conn
                .del(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(deleted > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_hash_all(&self, key: &str) -> Result<HashMap<String, String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.hgetall(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_list_range(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.lrange(key, start as isize, stop as isize)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_set_members(&self, key: &str) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.smembers(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_zset_range(&self, key: &str, start: i64, stop: i64) -> Result<Vec<(String, f64)>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let result: Vec<(String, f64)> = conn
                .zrange_withscores(key, start as isize, stop as isize)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(result)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn set_ttl(&self, key: &str, ttl: i64) -> Result<bool> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let result: i32 = conn
                .expire(key, ttl)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn remove_ttl(&self, key: &str) -> Result<bool> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let result: i32 = conn
                .persist(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn rename_key(&self, old_key: &str, new_key: &str) -> Result<()> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.rename(old_key, new_key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn hash_set_field(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.hset(key, field, value)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn hash_delete_field(&self, key: &str, field: &str) -> Result<bool> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let deleted: i32 = conn
                .hdel(key, field)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(deleted > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn execute_raw_command(&self, command: &str) -> Result<String> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                return Ok("Empty command".to_string());
            }
            
            let cmd_name = parts[0].to_uppercase();
            let args: Vec<&str> = parts[1..].to_vec();
            
            let mut redis_cmd = redis::cmd(&cmd_name);
            for arg in args {
                redis_cmd.arg(arg);
            }
            
            let result: redis::Value = redis_cmd
                .query_async(conn)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            let formatted = format_redis_value(&result);
            Ok(formatted)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let info: String = redis::cmd("INFO")
                .query_async(conn)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(parse_server_info(&info))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_raw_info(&self) -> Result<String> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let info: String = redis::cmd("INFO")
                .query_async(conn)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(info)
        } else {
            Err(ConnectionError::Closed)
        }
    }
}

fn format_redis_value(value: &redis::Value) -> String {
    match value {
        redis::Value::Nil => "(nil)".to_string(),
        redis::Value::Int(i) => format!("(integer) {}", i),
        redis::Value::BulkString(data) => {
            match String::from_utf8(data.clone()) {
                Ok(s) => format!("\"{}\"", s),
                Err(_) => format!("{:?}", data),
            }
        }
        redis::Value::Array(items) => {
            if items.is_empty() {
                "(empty list or set)".to_string()
            } else {
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| format!("{}) {}", i + 1, format_redis_value(item)))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        redis::Value::SimpleString(s) => s.clone(),
        redis::Value::Okay => "OK".to_string(),
        _ => format!("{:?}", value),
    }
}

fn parse_server_info(info: &str) -> ServerInfo {
    let mut server_info = ServerInfo::default();
    
    for line in info.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            
            match key {
                "redis_version" => server_info.redis_version = Some(value.to_string()),
                "redis_mode" => server_info.redis_mode = Some(value.to_string()),
                "os" => server_info.os = Some(value.to_string()),
                "arch_bits" => server_info.arch_bits = Some(value.to_string()),
                "process_id" => server_info.process_id = value.parse().ok(),
                "tcp_port" => server_info.tcp_port = value.parse().ok(),
                "uptime_in_seconds" => {
                    if let Some(secs) = value.parse::<u64>().ok() {
                        server_info.uptime_in_seconds = Some(secs);
                        server_info.uptime_in_days = Some(secs / 86400);
                    }
                }
                "connected_clients" => server_info.connected_clients = value.parse().ok(),
                "maxclients" => server_info.max_clients = value.parse().ok(),
                "total_connections_received" => server_info.total_connections_received = value.parse().ok(),
                "total_commands_processed" => server_info.total_commands_processed = value.parse().ok(),
                "instantaneous_ops_per_sec" => server_info.instantaneous_ops_per_sec = value.parse().ok(),
                "total_net_input_bytes" => server_info.total_net_input_bytes = value.parse().ok(),
                "total_net_output_bytes" => server_info.total_net_output_bytes = value.parse().ok(),
                "used_memory" => server_info.used_memory = value.parse().ok(),
                "used_memory_human" => server_info.used_memory_human = Some(value.to_string()),
                "used_memory_peak" => server_info.used_memory_peak = value.parse().ok(),
                "used_memory_peak_human" => server_info.used_memory_peak_human = Some(value.to_string()),
                "used_memory_rss" => server_info.used_memory_rss = value.parse().ok(),
                "mem_fragmentation_ratio" => server_info.mem_fragmentation_ratio = value.parse().ok(),
                "mem_allocator" => server_info.mem_allocator = Some(value.to_string()),
                "rdb_last_save_time" => server_info.rdb_last_save_time = value.parse().ok(),
                "rdb_changes_since_last_save" => server_info.rdb_changes_since_last_save = value.parse().ok(),
                "aof_enabled" => server_info.aof_enabled = value.parse().ok(),
                "aof_rewrite_in_progress" => server_info.aof_rewrite_in_progress = value.parse().ok(),
                key if key.starts_with("db") => {
                    if let Some(stats) = parse_db_stats(value) {
                        server_info.keyspace.insert(key.to_string(), stats.keys);
                        server_info.keys_total += stats.keys;
                        server_info.expires_total += stats.expires;
                    }
                }
                _ => {}
            }
        }
    }
    
    server_info
}

struct DbStats {
    keys: u64,
    expires: u64,
}

fn parse_db_stats(value: &str) -> Option<DbStats> {
    let mut keys = 0u64;
    let mut expires = 0u64;
    
    for part in value.split(',') {
        if let Some((k, v)) = part.split_once('=') {
            match k.trim() {
                "keys" => keys = v.parse().ok()?,
                "expires" => expires = v.parse().ok()?,
                _ => {}
            }
        }
    }
    
    Some(DbStats { keys, expires })
}

impl ConnectionPool {
    pub async fn set_hash_values(&self, key: &str, fields: Vec<HashField>) -> Result<()> {
        if fields.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "Hash fields cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            for field in fields {
                conn.hset::<_, _, _, ()>(key, field.field.clone(), field.value.clone())
                    .await
                    .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            }
            Ok(())
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_list_values(&self, key: &str, values: Vec<ListValue>) -> Result<()> {
        if values.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "List values cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let items: Vec<String> = values.into_iter().map(|v| v.value).collect();
            conn.rpush(key, items)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_set_values(&self, key: &str, values: Vec<SetValue>) -> Result<()> {
        if values.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "Set values cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let items: Vec<String> = values.into_iter().map(|v| v.value).collect();
            conn.sadd(key, items)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_zset_members(&self, key: &str, members: Vec<ZSetMember>) -> Result<()> {
        if members.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "ZSet members cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            for member in members {
                let score: f64 = member.score.parse().unwrap_or(0.0);
                conn.zadd::<_, _, _, ()>(key, member.value.clone(), score)
                    .await
                    .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            }
            Ok(())
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn add_stream_entries(&self, key: &str, entries: Vec<StreamEntry>) -> Result<()> {
        if entries.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "Stream entries cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            for entry in entries {
                let id = if entry.id.is_empty() || entry.id == "*" {
                    "*"
                } else {
                    &entry.id
                };
                redis::cmd("XADD")
                    .arg(key)
                    .arg(id)
                    .arg(&entry.field)
                    .arg(&entry.value)
                    .query_async::<redis::Value>(conn)
                    .await
                    .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            }
            Ok(())
        } else {
            Err(ConnectionError::Closed)
        }
    }
}