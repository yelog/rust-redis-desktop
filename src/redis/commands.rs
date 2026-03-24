use super::cluster::{
    parse_cluster_info, parse_cluster_nodes, parse_sentinel_masters, ClusterInfo, ClusterNode,
    SentinelInfo,
};
use super::{KeyInfo, KeyType};
use crate::connection::{ConnectionError, ConnectionPool, RedisConnection, Result};
use crate::ui::add_key_dialog::{HashField, ListValue, SetValue, StreamEntry, ZSetMember};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum LargeKeyData {
    String {
        value: String,
    },
    Hash {
        items: Vec<(String, String)>,
        cursor: u64,
        total: u64,
    },
    List {
        items: Vec<String>,
        total: u64,
    },
    Set {
        items: Vec<String>,
        cursor: u64,
        total: u64,
    },
    ZSet {
        items: Vec<(String, f64)>,
        cursor: u64,
        total: u64,
    },
    Stream {
        entries: Vec<(String, Vec<(String, String)>)>,
    },
}

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
                let result: (u64, Vec<String>) = conn
                    .execute_cmd(
                        &mut redis::cmd("SCAN")
                            .arg(cursor)
                            .arg("MATCH")
                            .arg(pattern)
                            .arg("COUNT")
                            .arg(count),
                    )
                    .await?;

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

    pub async fn scan_keys_with_cursor(
        &self,
        pattern: &str,
        cursor: u64,
        count: usize,
    ) -> Result<(u64, Vec<String>)> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: (u64, Vec<String>) = conn
                .execute_cmd(
                    redis::cmd("SCAN")
                        .arg(cursor)
                        .arg("MATCH")
                        .arg(pattern)
                        .arg("COUNT")
                        .arg(count),
                )
                .await?;

            Ok(result)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn scan_keys_with_progress<F>(
        &self,
        pattern: &str,
        batch_size: usize,
        mut on_batch: F,
    ) -> Result<usize>
    where
        F: FnMut(usize) + Send,
    {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let mut total_count = 0;
            let mut cursor: u64 = 0;

            loop {
                let result: (u64, Vec<String>) = conn
                    .execute_cmd(
                        redis::cmd("SCAN")
                            .arg(cursor)
                            .arg("MATCH")
                            .arg(pattern)
                            .arg("COUNT")
                            .arg(batch_size),
                    )
                    .await?;

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
            let type_str: String = conn.execute_cmd(&mut redis::cmd("TYPE").arg(key)).await?;
            Ok(KeyType::from(type_str))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_key_types(&self, keys: &[String]) -> Result<HashMap<String, KeyType>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let mut result = HashMap::new();

            const BATCH_SIZE: usize = 100;

            for chunk in keys.chunks(BATCH_SIZE) {
                let types: Vec<String> = match conn {
                    RedisConnection::Single(ref mut c) => {
                        let mut pipe = redis::pipe();
                        for key in chunk {
                            pipe.cmd("TYPE").arg(key);
                        }
                        pipe.query_async(c)
                            .await
                            .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?
                    }
                    RedisConnection::Cluster(ref mut c) => {
                        let mut types = Vec::with_capacity(chunk.len());
                        for key in chunk {
                            let type_str: String = redis::cmd("TYPE")
                                .arg(key)
                                .query_async(c)
                                .await
                                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
                            types.push(type_str);
                        }
                        types
                    }
                };

                for (key, type_str) in chunk.iter().zip(types.iter()) {
                    result.insert(key.clone(), KeyType::from(type_str.clone()));
                }
            }

            Ok(result)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_key_info(&self, key: &str) -> Result<KeyInfo> {
        let key_type = self.get_key_type(key).await?;

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let ttl: i64 = conn.ttl(key).await?;

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
            conn.get_string(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_string_bytes(&self, key: &str) -> Result<Vec<u8>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.get_bytes(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_string_value(&self, key: &str, value: &str) -> Result<()> {
        self.check_write_permission("SET")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.set_string(key, value).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn delete_key(&self, key: &str) -> Result<bool> {
        self.check_write_permission("DEL")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let deleted: i32 = conn.del_key(key).await?;
            Ok(deleted > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_hash_all(&self, key: &str) -> Result<HashMap<String, String>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.hgetall(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_list_range(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.lrange(key, start as isize, stop as isize).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_set_members(&self, key: &str) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.smembers(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_zset_range(
        &self,
        key: &str,
        start: i64,
        stop: i64,
    ) -> Result<Vec<(String, f64)>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.zrange_withscores(key, start as isize, stop as isize)
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_ttl(&self, key: &str, ttl: i64) -> Result<bool> {
        self.check_write_permission("EXPIRE")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.expire(key, ttl).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn remove_ttl(&self, key: &str) -> Result<bool> {
        self.check_write_permission("PERSIST")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.persist(key).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn rename_key(&self, old_key: &str, new_key: &str) -> Result<()> {
        self.check_write_permission("RENAME")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.rename(old_key, new_key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn hash_set_field(&self, key: &str, field: &str, value: &str) -> Result<()> {
        self.check_write_permission("HSET")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.hset(key, field, value).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn hash_delete_field(&self, key: &str, field: &str) -> Result<bool> {
        self.check_write_permission("HDEL")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let deleted: i32 = conn.hdel(key, field).await?;
            Ok(deleted > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn execute_raw_command(&self, command: &str) -> Result<String> {
        self.check_raw_command_permission(command)?;

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

            let result: redis::Value = conn.execute_cmd(&mut redis_cmd).await?;
            let formatted = format_redis_value(&result);
            Ok(formatted)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let info: String = conn.execute_cmd(&mut redis::cmd("INFO")).await?;
            Ok(parse_server_info(&info))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_raw_info(&self) -> Result<String> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let info: String = conn.execute_cmd(&mut redis::cmd("INFO")).await?;
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
        redis::Value::BulkString(data) => match String::from_utf8(data.clone()) {
            Ok(s) => format!("\"{}\"", s),
            Err(_) => format!("{:?}", data),
        },
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
                "total_connections_received" => {
                    server_info.total_connections_received = value.parse().ok()
                }
                "total_commands_processed" => {
                    server_info.total_commands_processed = value.parse().ok()
                }
                "instantaneous_ops_per_sec" => {
                    server_info.instantaneous_ops_per_sec = value.parse().ok()
                }
                "total_net_input_bytes" => server_info.total_net_input_bytes = value.parse().ok(),
                "total_net_output_bytes" => server_info.total_net_output_bytes = value.parse().ok(),
                "used_memory" => server_info.used_memory = value.parse().ok(),
                "used_memory_human" => server_info.used_memory_human = Some(value.to_string()),
                "used_memory_peak" => server_info.used_memory_peak = value.parse().ok(),
                "used_memory_peak_human" => {
                    server_info.used_memory_peak_human = Some(value.to_string())
                }
                "used_memory_rss" => server_info.used_memory_rss = value.parse().ok(),
                "mem_fragmentation_ratio" => {
                    server_info.mem_fragmentation_ratio = value.parse().ok()
                }
                "mem_allocator" => server_info.mem_allocator = Some(value.to_string()),
                "rdb_last_save_time" => server_info.rdb_last_save_time = value.parse().ok(),
                "rdb_changes_since_last_save" => {
                    server_info.rdb_changes_since_last_save = value.parse().ok()
                }
                "aof_enabled" => server_info.aof_enabled = value.parse().ok(),
                "aof_rewrite_in_progress" => {
                    server_info.aof_rewrite_in_progress = value.parse().ok()
                }
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
        self.check_write_permission("HSET")?;
        if fields.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "Hash fields cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            for field in fields {
                conn.hset(key, &field.field, &field.value).await?;
            }
            Ok(())
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_list_values(&self, key: &str, values: Vec<ListValue>) -> Result<()> {
        self.check_write_permission("RPUSH")?;
        if values.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "List values cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let items: Vec<String> = values.into_iter().map(|v| v.value).collect();
            conn.rpush_vec(key, items).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_set_values(&self, key: &str, values: Vec<SetValue>) -> Result<()> {
        self.check_write_permission("SADD")?;
        if values.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "Set values cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let items: Vec<String> = values.into_iter().map(|v| v.value).collect();
            conn.sadd_vec(key, items).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_zset_members(&self, key: &str, members: Vec<ZSetMember>) -> Result<()> {
        self.check_write_permission("ZADD")?;
        if members.is_empty() {
            return Err(ConnectionError::ConnectionFailed(
                "ZSet members cannot be empty".to_string(),
            ));
        }

        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            for member in members {
                let score: f64 = member.score.parse().unwrap_or(0.0);
                conn.zadd_str(key, score, &member.value).await?;
            }
            Ok(())
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn add_stream_entries(&self, key: &str, entries: Vec<StreamEntry>) -> Result<()> {
        self.check_write_permission("XADD")?;
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
                conn.execute_cmd::<redis::Value>(
                    redis::cmd("XADD")
                        .arg(key)
                        .arg(id)
                        .arg(&entry.field)
                        .arg(&entry.value),
                )
                .await?;
            }
            Ok(())
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn list_push(&self, key: &str, value: &str, left: bool) -> Result<i64> {
        self.check_write_permission(if left { "LPUSH" } else { "RPUSH" })?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i64 = if left {
                conn.lpush(key, value).await?
            } else {
                conn.rpush(key, value).await?
            };
            Ok(result)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn list_pop(&self, key: &str, left: bool) -> Result<Option<String>> {
        self.check_write_permission(if left { "LPOP" } else { "RPOP" })?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: Option<String> = if left {
                conn.lpop(key).await?
            } else {
                conn.rpop(key).await?
            };
            Ok(result)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn list_set(&self, key: &str, index: i64, value: &str) -> Result<()> {
        self.check_write_permission("LSET")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.lset(key, index as isize, value).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn list_remove(&self, key: &str, count: i64, value: &str) -> Result<i64> {
        self.check_write_permission("LREM")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.lrem(key, count as isize, value).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn list_len(&self, key: &str) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.llen(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_add(&self, key: &str, member: &str) -> Result<bool> {
        self.check_write_permission("SADD")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.sadd(key, member).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_remove(&self, key: &str, member: &str) -> Result<bool> {
        self.check_write_permission("SREM")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.srem(key, member).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn zset_add(&self, key: &str, member: &str, score: f64) -> Result<bool> {
        self.check_write_permission("ZADD")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.zadd(key, score, member).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn zset_remove(&self, key: &str, member: &str) -> Result<bool> {
        self.check_write_permission("ZREM")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.zrem(key, member).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn zset_card(&self, key: &str) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.zcard(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_delete(&self, key: &str, id: &str) -> Result<bool> {
        self.check_write_permission("XDEL")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn
                .execute_cmd(&mut redis::cmd("XDEL").arg(key).arg(id))
                .await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_len(&self, key: &str) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("XLEN").arg(key)).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_range(
        &self,
        key: &str,
        start: &str,
        end: &str,
    ) -> Result<Vec<(String, Vec<(String, String)>)>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("XRANGE").arg(key).arg(start).arg(end))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_create_group(
        &self,
        key: &str,
        group: &str,
        id: &str,
        mkstream: bool,
    ) -> Result<()> {
        self.check_write_permission("XGROUP")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.xgroup_create(key, group, id, mkstream).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_destroy_group(&self, key: &str, group: &str) -> Result<bool> {
        self.check_write_permission("XGROUP")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.xgroup_destroy(key, group).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_delete_consumer(
        &self,
        key: &str,
        group: &str,
        consumer: &str,
    ) -> Result<bool> {
        self.check_write_permission("XGROUP")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: i32 = conn.xgroup_delconsumer(key, group, consumer).await?;
            Ok(result > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_get_groups_raw(&self, key: &str) -> Result<redis::Value> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.xinfo_groups_raw(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn stream_get_consumers_raw(
        &self,
        key: &str,
        group: &str,
    ) -> Result<redis::Value> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.xinfo_consumers_raw(key, group).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn hash_len(&self, key: &str) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.hlen(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_len(&self, key: &str) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.scard(key).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_list_page(&self, key: &str, offset: i64, count: i64) -> Result<Vec<String>> {
        self.get_list_range(key, offset, offset + count - 1).await
    }

    pub async fn get_large_key_data(
        &self,
        key: &str,
        key_type: KeyType,
        offset: usize,
        count: usize,
    ) -> Result<LargeKeyData> {
        match key_type {
            KeyType::List => {
                let items = self.get_list_page(key, offset as i64, count as i64).await?;
                let total = self.list_len(key).await?;
                Ok(LargeKeyData::List { items, total })
            }
            KeyType::Set => {
                let (cursor, items) = self.get_set_page(key, offset as u64, count).await?;
                let total = self.set_len(key).await?;
                Ok(LargeKeyData::Set {
                    items,
                    cursor,
                    total,
                })
            }
            KeyType::ZSet => {
                let (cursor, items) = self.get_zset_page(key, offset as u64, count).await?;
                let total = self.zset_card(key).await?;
                Ok(LargeKeyData::ZSet {
                    items,
                    cursor,
                    total,
                })
            }
            KeyType::Hash => {
                let (cursor, items) = self.get_hash_page(key, offset as u64, count).await?;
                let total = self.hash_len(key).await?;
                Ok(LargeKeyData::Hash {
                    items,
                    cursor,
                    total,
                })
            }
            KeyType::String => {
                let value = self.get_string_value(key).await?;
                Ok(LargeKeyData::String { value })
            }
            KeyType::Stream => {
                let entries = self.stream_range(key, "-", "+").await?;
                Ok(LargeKeyData::Stream { entries })
            }
            KeyType::None => Err(ConnectionError::ConnectionFailed(
                "Key not found".to_string(),
            )),
        }
    }

    pub async fn get_hash_page(
        &self,
        key: &str,
        cursor: u64,
        count: usize,
    ) -> Result<(u64, Vec<(String, String)>)> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(
                &mut redis::cmd("HSCAN")
                    .arg(key)
                    .arg(cursor)
                    .arg("COUNT")
                    .arg(count),
            )
            .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_set_page(
        &self,
        key: &str,
        cursor: u64,
        count: usize,
    ) -> Result<(u64, Vec<String>)> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(
                &mut redis::cmd("SSCAN")
                    .arg(key)
                    .arg(cursor)
                    .arg("COUNT")
                    .arg(count),
            )
            .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_zset_page(
        &self,
        key: &str,
        cursor: u64,
        count: usize,
    ) -> Result<(u64, Vec<(String, f64)>)> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(
                &mut redis::cmd("ZSCAN")
                    .arg(key)
                    .arg(cursor)
                    .arg("COUNT")
                    .arg(count),
            )
            .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn memory_usage(&self, key: &str) -> Result<Option<u64>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("MEMORY").arg("USAGE").arg(key))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn flush_db(&self) -> Result<()> {
        self.check_write_permission("FLUSHDB")?;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd::<()>(&mut redis::cmd("FLUSHDB")).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn db_size(&self) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("DBSIZE")).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn dump_key(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: Option<Vec<u8>> =
                conn.execute_cmd(&mut redis::cmd("DUMP").arg(key)).await?;
            Ok(result)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn restore_key(&self, key: &str, ttl: i64, data: &[u8]) -> Result<()> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd::<()>(&mut redis::cmd("RESTORE").arg(key).arg(ttl).arg(data))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn restore_key_replace(&self, key: &str, ttl: i64, data: &[u8]) -> Result<()> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd::<()>(
                &mut redis::cmd("RESTORE")
                    .arg(key)
                    .arg(ttl)
                    .arg(data)
                    .arg("REPLACE"),
            )
            .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn import_json_data(&self, data: &str) -> Result<usize> {
        let import_data: Vec<ImportKeyData> = serde_json::from_str(data)
            .map_err(|e| ConnectionError::ConnectionFailed(format!("Invalid JSON: {}", e)))?;

        let mut imported = 0;
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            for key_data in import_data {
                let ttl = key_data.ttl.unwrap_or(-1);

                match key_data.key_type.as_str() {
                    "string" => {
                        if let Some(value) = key_data.value {
                            conn.execute_cmd::<()>(
                                &mut redis::cmd("SET").arg(&key_data.key).arg(&value),
                            )
                            .await?;
                            if ttl > 0 {
                                conn.execute_cmd::<()>(
                                    &mut redis::cmd("EXPIRE").arg(&key_data.key).arg(ttl),
                                )
                                .await?;
                            }
                            imported += 1;
                        }
                    }
                    "hash" => {
                        if let Some(fields) = key_data.fields {
                            for (field, value) in fields {
                                conn.execute_cmd::<()>(
                                    &mut redis::cmd("HSET")
                                        .arg(&key_data.key)
                                        .arg(&field)
                                        .arg(&value),
                                )
                                .await?;
                            }
                            if ttl > 0 {
                                conn.execute_cmd::<()>(
                                    &mut redis::cmd("EXPIRE").arg(&key_data.key).arg(ttl),
                                )
                                .await?;
                            }
                            imported += 1;
                        }
                    }
                    "list" => {
                        if let Some(elements) = key_data.elements {
                            if !elements.is_empty() {
                                conn.execute_cmd::<()>(
                                    &mut redis::cmd("RPUSH").arg(&key_data.key).arg(&elements),
                                )
                                .await?;
                                if ttl > 0 {
                                    conn.execute_cmd::<()>(
                                        &mut redis::cmd("EXPIRE").arg(&key_data.key).arg(ttl),
                                    )
                                    .await?;
                                }
                                imported += 1;
                            }
                        }
                    }
                    "set" => {
                        if let Some(members) = key_data.members {
                            if !members.is_empty() {
                                conn.execute_cmd::<()>(
                                    &mut redis::cmd("SADD").arg(&key_data.key).arg(&members),
                                )
                                .await?;
                                if ttl > 0 {
                                    conn.execute_cmd::<()>(
                                        &mut redis::cmd("EXPIRE").arg(&key_data.key).arg(ttl),
                                    )
                                    .await?;
                                }
                                imported += 1;
                            }
                        }
                    }
                    "zset" => {
                        if let Some(members) = key_data.scored_members {
                            for (member, score) in members {
                                if let Ok(s) = score.parse::<f64>() {
                                    conn.execute_cmd::<()>(
                                        &mut redis::cmd("ZADD")
                                            .arg(&key_data.key)
                                            .arg(s)
                                            .arg(&member),
                                    )
                                    .await?;
                                }
                            }
                            if ttl > 0 {
                                conn.execute_cmd::<()>(
                                    &mut redis::cmd("EXPIRE").arg(&key_data.key).arg(ttl),
                                )
                                .await?;
                            }
                            imported += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(imported)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ImportKeyData {
    pub key: String,
    #[serde(rename = "type")]
    pub key_type: String,
    pub ttl: Option<i64>,
    pub value: Option<String>,
    pub fields: Option<HashMap<String, String>>,
    pub elements: Option<Vec<String>>,
    pub members: Option<Vec<String>>,
    pub scored_members: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportKeyData {
    pub key: String,
    #[serde(rename = "type")]
    pub key_type: String,
    pub ttl: Option<i64>,
    pub value: Option<String>,
    pub fields: Option<HashMap<String, String>>,
    pub elements: Option<Vec<String>>,
    pub members: Option<Vec<String>>,
    pub scored_members: Option<Vec<(String, f64)>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Commands,
}

impl ConnectionPool {
    pub async fn export_keys(&self, keys: &[String], format: ExportFormat) -> Result<String> {
        let mut exported_keys = Vec::new();

        for key in keys {
            if let Ok(key_info) = self.get_key_info(key).await {
                let ttl = key_info.ttl;

                let export_data = match key_info.key_type {
                    KeyType::String => {
                        let value = self.get_string_value(key).await.ok();
                        ExportKeyData {
                            key: key.clone(),
                            key_type: "string".to_string(),
                            ttl,
                            value,
                            fields: None,
                            elements: None,
                            members: None,
                            scored_members: None,
                        }
                    }
                    KeyType::Hash => {
                        let fields = self.get_hash_all(key).await.ok();
                        ExportKeyData {
                            key: key.clone(),
                            key_type: "hash".to_string(),
                            ttl,
                            value: None,
                            fields,
                            elements: None,
                            members: None,
                            scored_members: None,
                        }
                    }
                    KeyType::List => {
                        let elements = self.get_list_range(key, 0, -1).await.ok();
                        ExportKeyData {
                            key: key.clone(),
                            key_type: "list".to_string(),
                            ttl,
                            value: None,
                            fields: None,
                            elements,
                            members: None,
                            scored_members: None,
                        }
                    }
                    KeyType::Set => {
                        let members = self.get_set_members(key).await.ok();
                        ExportKeyData {
                            key: key.clone(),
                            key_type: "set".to_string(),
                            ttl,
                            value: None,
                            fields: None,
                            elements: None,
                            members,
                            scored_members: None,
                        }
                    }
                    KeyType::ZSet => {
                        let scored_members = self.get_zset_range(key, 0, -1).await.ok();
                        ExportKeyData {
                            key: key.clone(),
                            key_type: "zset".to_string(),
                            ttl,
                            value: None,
                            fields: None,
                            elements: None,
                            members: None,
                            scored_members,
                        }
                    }
                    KeyType::Stream => ExportKeyData {
                        key: key.clone(),
                        key_type: "stream".to_string(),
                        ttl,
                        value: None,
                        fields: None,
                        elements: None,
                        members: None,
                        scored_members: None,
                    },
                    KeyType::None => continue,
                };

                exported_keys.push(export_data);
            }
        }

        match format {
            ExportFormat::Json => serde_json::to_string_pretty(&exported_keys)
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string())),
            ExportFormat::Commands => {
                let mut commands = String::new();
                for data in exported_keys {
                    let key = &data.key;

                    match data.key_type.as_str() {
                        "string" => {
                            if let Some(v) = &data.value {
                                commands.push_str(&format!("SET {} {}\n", key, escape_value(v)));
                            }
                        }
                        "hash" => {
                            if let Some(fields) = &data.fields {
                                for (field, value) in fields {
                                    commands.push_str(&format!(
                                        "HSET {} {} {}\n",
                                        key,
                                        field,
                                        escape_value(value)
                                    ));
                                }
                            }
                        }
                        "list" => {
                            if let Some(elements) = &data.elements {
                                if !elements.is_empty() {
                                    let values: Vec<&str> =
                                        elements.iter().map(|s| s.as_str()).collect();
                                    commands.push_str(&format!(
                                        "RPUSH {} {}\n",
                                        key,
                                        values.join(" ")
                                    ));
                                }
                            }
                        }
                        "set" => {
                            if let Some(members) = &data.members {
                                if !members.is_empty() {
                                    let values: Vec<&str> =
                                        members.iter().map(|s| s.as_str()).collect();
                                    commands.push_str(&format!(
                                        "SADD {} {}\n",
                                        key,
                                        values.join(" ")
                                    ));
                                }
                            }
                        }
                        "zset" => {
                            if let Some(scored_members) = &data.scored_members {
                                for (member, score) in scored_members {
                                    commands
                                        .push_str(&format!("ZADD {} {} {}\n", key, score, member));
                                }
                            }
                        }
                        _ => {}
                    }

                    if let Some(ttl) = data.ttl {
                        if ttl > 0 {
                            commands.push_str(&format!("EXPIRE {} {}\n", key, ttl));
                        }
                    }
                }
                Ok(commands)
            }
        }
    }

    pub async fn export_all_keys(&self, pattern: &str, format: ExportFormat) -> Result<String> {
        let keys = self.scan_keys(pattern, 1000).await?;
        self.export_keys(&keys, format).await
    }
}

fn escape_value(value: &str) -> String {
    if value.contains(' ') || value.contains('\n') || value.contains('"') {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

impl ConnectionPool {
    pub async fn eval_script(
        &self,
        script: &str,
        keys: &[String],
        args: &[String],
    ) -> Result<redis::Value> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let mut cmd = redis::cmd("EVAL");
            cmd.arg(script).arg(keys.len() as i32);
            for key in keys {
                cmd.arg(key);
            }
            for arg in args {
                cmd.arg(arg);
            }
            conn.execute_cmd(&mut cmd).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn evalsha(
        &self,
        sha: &str,
        keys: &[String],
        args: &[String],
    ) -> Result<redis::Value> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let mut cmd = redis::cmd("EVALSHA");
            cmd.arg(sha).arg(keys.len() as i32);
            for key in keys {
                cmd.arg(key);
            }
            for arg in args {
                cmd.arg(arg);
            }
            conn.execute_cmd(&mut cmd).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn script_load(&self, script: &str) -> Result<String> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("SCRIPT").arg("LOAD").arg(script))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn script_exists(&self, sha: &[String]) -> Result<Vec<bool>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let mut cmd = redis::cmd("SCRIPT");
            cmd.arg("EXISTS");
            for s in sha {
                cmd.arg(s);
            }
            conn.execute_cmd(&mut cmd).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn script_flush(&self) -> Result<()> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd::<()>(&mut redis::cmd("SCRIPT").arg("FLUSH"))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }
}

impl ConnectionPool {
    pub async fn get_cluster_nodes(&self) -> Result<Vec<ClusterNode>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let output: String = conn
                .execute_cmd(&mut redis::cmd("CLUSTER").arg("NODES"))
                .await?;
            Ok(parse_cluster_nodes(&output))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let output: String = conn
                .execute_cmd(&mut redis::cmd("CLUSTER").arg("INFO"))
                .await?;
            Ok(parse_cluster_info(&output))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_cluster_slots(&self) -> Result<Vec<redis::Value>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("CLUSTER").arg("SLOTS"))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_sentinel_masters(&self) -> Result<Vec<SentinelInfo>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let output: String = conn
                .execute_cmd(&mut redis::cmd("SENTINEL").arg("MASTERS"))
                .await?;
            Ok(parse_sentinel_masters(&output))
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_sentinel_master(&self, name: &str) -> Result<HashMap<String, String>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("SENTINEL").arg("MASTER").arg(name))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_sentinel_replicas(&self, name: &str) -> Result<Vec<HashMap<String, String>>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("SENTINEL").arg("REPLICAS").arg(name))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_sentinel_sentinels(&self, name: &str) -> Result<Vec<HashMap<String, String>>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("SENTINEL").arg("SENTINELS").arg(name))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn sentinel_get_master_addr_by_name(
        &self,
        name: &str,
    ) -> Result<Option<(String, u16)>> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: Vec<String> = conn
                .execute_cmd(
                    &mut redis::cmd("SENTINEL")
                        .arg("GET-MASTER-ADDR-BY-NAME")
                        .arg(name),
                )
                .await?;

            if result.len() >= 2 {
                let host = result[0].clone();
                let port: u16 = result[1].parse().unwrap_or(6379);
                Ok(Some((host, port)))
            } else {
                Ok(None)
            }
        } else {
            Err(ConnectionError::Closed)
        }
    }
}

impl ConnectionPool {
    pub async fn get_bit(&self, key: &str, offset: u64) -> Result<bool> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: u8 = conn
                .execute_cmd(&mut redis::cmd("GETBIT").arg(key).arg(offset))
                .await?;
            Ok(result == 1)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn set_bit(&self, key: &str, offset: u64, value: bool) -> Result<bool> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            let result: u8 = conn
                .execute_cmd(
                    &mut redis::cmd("SETBIT")
                        .arg(key)
                        .arg(offset)
                        .arg(if value { 1 } else { 0 }),
                )
                .await?;
            Ok(result == 1)
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn bit_count(&self, key: &str) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("BITCOUNT").arg(key)).await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn bit_count_range(&self, key: &str, start: i64, end: i64) -> Result<u64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("BITCOUNT").arg(key).arg(start).arg(end))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn bit_pos(&self, key: &str, bit: bool) -> Result<i64> {
        let mut connection = self.connection.lock().await;

        if let Some(ref mut conn) = *connection {
            conn.execute_cmd(&mut redis::cmd("BITPOS").arg(key).arg(if bit { 1 } else { 0 }))
                .await
        } else {
            Err(ConnectionError::Closed)
        }
    }

    pub async fn get_bitmap_info(&self, key: &str) -> Result<BitmapInfo> {
        let bytes = self.get_string_bytes(key).await?;
        let bit_count = self.bit_count(key).await?;

        let set_bits: Vec<u64> = {
            let mut bits = Vec::new();
            for (byte_idx, &byte) in bytes.iter().enumerate() {
                for bit_idx in 0..8 {
                    if (byte >> (7 - bit_idx)) & 1 == 1 {
                        bits.push((byte_idx as u64 * 8 + bit_idx as u64) as u64);
                    }
                }
            }
            bits
        };

        Ok(BitmapInfo {
            total_bytes: bytes.len() as u64,
            total_bits: bytes.len() as u64 * 8,
            set_bits_count: bit_count,
            set_bits,
            raw_bytes: bytes,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitmapInfo {
    pub total_bytes: u64,
    pub total_bits: u64,
    pub set_bits_count: u64,
    pub set_bits: Vec<u64>,
    pub raw_bytes: Vec<u8>,
}
