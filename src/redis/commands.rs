use super::{KeyInfo, KeyType};
use crate::connection::{ConnectionError, ConnectionPool, Result};
use redis::AsyncCommands;
use std::collections::HashMap;

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