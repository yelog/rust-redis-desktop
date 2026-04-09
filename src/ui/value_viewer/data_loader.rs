use super::formatters::{detect_image_format, format_bytes, is_binary_data};
use super::BinaryFormat;
use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType};
use crate::serialization::{detect_serialization_format, SerializationFormat};
use dioxus::prelude::*;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
pub(super) async fn load_key_data(
    pool: ConnectionPool,
    key: String,
    mut key_info: Signal<Option<KeyInfo>>,
    mut string_value: Signal<String>,
    mut hash_value: Signal<HashMap<String, String>>,
    mut list_value: Signal<Vec<String>>,
    mut set_value: Signal<Vec<String>>,
    mut zset_value: Signal<Vec<(String, f64)>>,
    mut stream_value: Signal<Vec<(String, Vec<(String, String)>)>>,
    mut is_binary: Signal<bool>,
    mut binary_format: Signal<BinaryFormat>,
    mut serialization_data: Signal<Option<(SerializationFormat, Vec<u8>)>>,
    mut binary_bytes: Signal<Vec<u8>>,
    mut bitmap_info: Signal<Option<crate::redis::BitmapInfo>>,
    mut loading: Signal<bool>,
    mut hash_cursor: Signal<u64>,
    mut hash_total: Signal<usize>,
    mut hash_has_more: Signal<bool>,
    mut list_has_more: Signal<bool>,
    mut list_total: Signal<usize>,
    mut set_cursor: Signal<u64>,
    mut set_total: Signal<usize>,
    mut set_has_more: Signal<bool>,
    mut zset_cursor: Signal<u64>,
    mut zset_total: Signal<usize>,
    mut zset_has_more: Signal<bool>,
) -> Result<(), String> {
    if key.is_empty() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        list_value.set(Vec::new());
        set_value.set(Vec::new());
        zset_value.set(Vec::new());
        stream_value.set(Vec::new());
        is_binary.set(false);
        serialization_data.set(None);
        bitmap_info.set(None);
        loading.set(false);
        return Ok(());
    }

    loading.set(true);

    let load_result = async {
        let info = pool
            .get_key_info(&key)
            .await
            .map_err(|e| format!("Failed to load key info: {e}"))?;

        tracing::info!("Key info loaded: {:?}", info.key_type);
        key_info.set(Some(info.clone()));

        match info.key_type {
            KeyType::String => {
                let bytes = pool
                    .get_string_bytes(&key)
                    .await
                    .map_err(|e| format!("Failed to load string value: {e}"))?;

                tracing::info!("String value loaded: {} bytes", bytes.len());

                if bytes.len() >= 4 {
                    tracing::info!("First 10 bytes: {:02x?}", &bytes[..10.min(bytes.len())]);
                }

                if is_binary_data(&bytes) {
                    is_binary.set(true);
                    binary_bytes.set(bytes.clone());

                    if detect_image_format(&bytes).is_some() {
                        serialization_data.set(None);
                        binary_format.set(BinaryFormat::Image);
                    } else {
                        let detected_format = detect_serialization_format(&bytes);
                        if detected_format != SerializationFormat::Unknown {
                            tracing::info!("Detected serialization format: {:?}", detected_format);
                            serialization_data.set(Some((detected_format, bytes.clone())));
                            binary_format.set(match detected_format {
                                SerializationFormat::Java => BinaryFormat::JavaSerialized,
                                SerializationFormat::Php => BinaryFormat::Php,
                                SerializationFormat::MsgPack => BinaryFormat::MsgPack,
                                SerializationFormat::Pickle => BinaryFormat::Pickle,
                                SerializationFormat::Kryo => BinaryFormat::Kryo,
                                SerializationFormat::Fst => BinaryFormat::Kryo,
                                SerializationFormat::Protobuf => BinaryFormat::Protobuf,
                                SerializationFormat::Bson => BinaryFormat::Bson,
                                SerializationFormat::Cbor => BinaryFormat::Cbor,
                                _ => BinaryFormat::Hex,
                            });
                        } else {
                            serialization_data.set(None);
                            if bytes.len() <= 1024 {
                                if let Ok(info) = pool.get_bitmap_info(&key).await {
                                    if info.set_bits_count > 0 {
                                        bitmap_info.set(Some(info));
                                        binary_format.set(BinaryFormat::Bitmap);
                                    }
                                }
                            }
                        }
                    }

                    let formatted = format_bytes(&bytes, binary_format());
                    string_value.set(formatted);
                } else {
                    binary_bytes.set(Vec::new());
                    is_binary.set(false);
                    serialization_data.set(None);
                    match String::from_utf8(bytes.clone()) {
                        Ok(s) => string_value.set(s),
                        Err(_) => {
                            is_binary.set(true);
                            binary_bytes.set(bytes.clone());

                            if detect_image_format(&bytes).is_some() {
                                binary_format.set(BinaryFormat::Image);
                            } else {
                                let detected_format = detect_serialization_format(&bytes);
                                if detected_format != SerializationFormat::Unknown {
                                    serialization_data.set(Some((detected_format, bytes.clone())));
                                    binary_format.set(match detected_format {
                                        SerializationFormat::Java => BinaryFormat::JavaSerialized,
                                        SerializationFormat::Php => BinaryFormat::Php,
                                        SerializationFormat::MsgPack => BinaryFormat::MsgPack,
                                        SerializationFormat::Pickle => BinaryFormat::Pickle,
                                        SerializationFormat::Kryo => BinaryFormat::Kryo,
                                        SerializationFormat::Fst => BinaryFormat::Kryo,
                                        SerializationFormat::Protobuf => BinaryFormat::Protobuf,
                                        SerializationFormat::Bson => BinaryFormat::Bson,
                                        SerializationFormat::Cbor => BinaryFormat::Cbor,
                                        _ => BinaryFormat::Hex,
                                    });
                                } else if bytes.len() <= 1024 {
                                    if let Ok(info) = pool.get_bitmap_info(&key).await {
                                        if info.set_bits_count > 0 {
                                            bitmap_info.set(Some(info));
                                            binary_format.set(BinaryFormat::Bitmap);
                                        }
                                    }
                                }
                            }

                            string_value.set(format_bytes(&bytes, binary_format()));
                        }
                    }
                }
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                list_has_more.set(false);
                list_total.set(0);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
            }
            KeyType::Hash => {
                let total = pool
                    .hash_len(&key)
                    .await
                    .map_err(|e| format!("Failed to load hash length: {e}"))?;
                let (cursor, items) = pool
                    .get_hash_page(&key, 0, super::PAGE_SIZE)
                    .await
                    .map_err(|e| format!("Failed to load hash data: {e}"))?;
                let fields: HashMap<String, String> = items.into_iter().collect();
                tracing::info!("Hash loaded: {} fields (total: {})", fields.len(), total);
                hash_value.set(fields);
                hash_cursor.set(cursor);
                hash_total.set(total as usize);
                hash_has_more.set(cursor != 0);
                string_value.set(String::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                list_has_more.set(false);
                list_total.set(0);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::List => {
                let total = pool
                    .list_len(&key)
                    .await
                    .map_err(|e| format!("Failed to load list length: {e}"))?;
                let count = super::PAGE_SIZE.min(total as usize);
                let items = if count == 0 {
                    Vec::new()
                } else {
                    pool.get_list_range(&key, 0, (count - 1) as i64)
                        .await
                        .map_err(|e| format!("Failed to load list data: {e}"))?
                };
                tracing::info!("List loaded: {} items (total: {})", items.len(), total);
                list_value.set(items.clone());
                list_has_more.set(items.len() == super::PAGE_SIZE && items.len() < total as usize);
                list_total.set(total as usize);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                stream_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::Set => {
                let total = pool
                    .set_len(&key)
                    .await
                    .map_err(|e| format!("Failed to load set length: {e}"))?;
                let (cursor, items) = pool
                    .get_set_page(&key, 0, super::PAGE_SIZE)
                    .await
                    .map_err(|e| format!("Failed to load set data: {e}"))?;
                tracing::info!("Set loaded: {} members (total: {})", items.len(), total);
                set_value.set(items);
                set_cursor.set(cursor);
                set_total.set(total as usize);
                set_has_more.set(cursor != 0);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                zset_value.set(Vec::new());
                stream_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                list_has_more.set(false);
                list_total.set(0);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::ZSet => {
                let total = pool
                    .zset_card(&key)
                    .await
                    .map_err(|e| format!("Failed to load zset length: {e}"))?;
                let (cursor, items) = pool
                    .get_zset_page(&key, 0, super::PAGE_SIZE)
                    .await
                    .map_err(|e| format!("Failed to load zset data: {e}"))?;
                tracing::info!("ZSet loaded: {} members (total: {})", items.len(), total);
                zset_value.set(items);
                zset_cursor.set(cursor);
                zset_total.set(total as usize);
                zset_has_more.set(cursor != 0);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                stream_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                list_has_more.set(false);
                list_total.set(0);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::Stream => {
                let entries = pool
                    .stream_range(&key, "-", "+")
                    .await
                    .map_err(|e| format!("Failed to load stream data: {e}"))?;
                tracing::info!("Stream loaded: {} entries", entries.len());
                stream_value.set(entries);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                is_binary.set(false);
                serialization_data.set(None);
            }
            _ => {
                tracing::info!("Type: {:?}", info.key_type);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                stream_value.set(Vec::new());
                is_binary.set(false);
                serialization_data.set(None);
            }
        }

        Ok::<(), String>(())
    }
    .await;

    if load_result.is_err() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        list_value.set(Vec::new());
        set_value.set(Vec::new());
        zset_value.set(Vec::new());
        stream_value.set(Vec::new());
        is_binary.set(false);
        serialization_data.set(None);
    }

    loading.set(false);
    load_result
}

pub(super) async fn load_more_hash(
    pool: ConnectionPool,
    key: String,
    mut hash_value: Signal<HashMap<String, String>>,
    cursor: u64,
    mut hash_cursor: Signal<u64>,
    mut hash_has_more: Signal<bool>,
    mut hash_loading_more: Signal<bool>,
    _hash_total: Signal<usize>,
) {
    if hash_loading_more() || !hash_has_more() {
        return;
    }
    hash_loading_more.set(true);
    match pool.get_hash_page(&key, cursor, super::PAGE_SIZE).await {
        Ok((new_cursor, items)) => {
            let mut current = hash_value();
            for (field, value) in items {
                current.insert(field, value);
            }
            hash_value.set(current);
            hash_cursor.set(new_cursor);
            hash_has_more.set(new_cursor != 0);
            hash_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 hash 数据失败: {}", e);
            hash_loading_more.set(false);
        }
    }
}

pub(super) async fn load_more_zset(
    pool: ConnectionPool,
    key: String,
    mut zset_value: Signal<Vec<(String, f64)>>,
    cursor: u64,
    mut zset_cursor: Signal<u64>,
    mut zset_has_more: Signal<bool>,
    mut zset_loading_more: Signal<bool>,
) {
    if zset_loading_more() || !zset_has_more() {
        return;
    }
    zset_loading_more.set(true);
    match pool.get_zset_page(&key, cursor, super::PAGE_SIZE).await {
        Ok((new_cursor, items)) => {
            let mut current = zset_value();
            current.extend(items);
            zset_value.set(current);
            zset_cursor.set(new_cursor);
            zset_has_more.set(new_cursor != 0);
            zset_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 zset 数据失败: {}", e);
            zset_loading_more.set(false);
        }
    }
}

pub(super) async fn load_more_set(
    pool: ConnectionPool,
    key: String,
    mut set_value: Signal<Vec<String>>,
    cursor: u64,
    mut set_cursor: Signal<u64>,
    mut set_has_more: Signal<bool>,
    mut set_loading_more: Signal<bool>,
) {
    if set_loading_more() || !set_has_more() {
        return;
    }
    set_loading_more.set(true);
    match pool.get_set_page(&key, cursor, super::PAGE_SIZE).await {
        Ok((new_cursor, items)) => {
            let mut current = set_value();
            current.extend(items);
            set_value.set(current);
            set_cursor.set(new_cursor);
            set_has_more.set(new_cursor != 0);
            set_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 set 数据失败: {}", e);
            set_loading_more.set(false);
        }
    }
}

pub(super) async fn load_more_list(
    pool: ConnectionPool,
    key: String,
    mut list_value: Signal<Vec<String>>,
    mut list_has_more: Signal<bool>,
    mut list_loading_more: Signal<bool>,
    total: usize,
) {
    if list_loading_more() || !list_has_more() {
        return;
    }
    list_loading_more.set(true);
    let offset = list_value().len() as i64;
    match pool
        .get_list_range(&key, offset, offset + super::PAGE_SIZE as i64 - 1)
        .await
    {
        Ok(items) => {
            let mut current = list_value();
            current.extend(items.clone());
            list_value.set(current);
            list_has_more.set(items.len() == super::PAGE_SIZE && list_value().len() < total);
            list_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 list 数据失败: {}", e);
            list_loading_more.set(false);
        }
    }
}

pub(super) async fn search_hash_server(
    pool: ConnectionPool,
    key: String,
    pattern: String,
    mut hash_value: Signal<HashMap<String, String>>,
    mut hash_cursor: Signal<u64>,
    mut hash_has_more: Signal<bool>,
    mut hash_loading_more: Signal<bool>,
) {
    hash_loading_more.set(true);
    let redis_pattern = if pattern.is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", pattern)
    };
    let mut cursor: u64 = 0;
    let mut all_items: HashMap<String, String> = HashMap::new();
    let max_iterations = 1000;
    let mut iterations = 0;
    loop {
        match pool
            .hash_scan_match(&key, &redis_pattern, cursor, super::PAGE_SIZE)
            .await
        {
            Ok((new_cursor, items)) => {
                for (field, value) in items {
                    all_items.insert(field, value);
                }
                cursor = new_cursor;
                iterations += 1;
                if cursor == 0 || iterations >= max_iterations {
                    break;
                }
                if !all_items.is_empty() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("搜索 hash 数据失败: {}", e);
                hash_loading_more.set(false);
                return;
            }
        }
    }
    hash_value.set(all_items);
    hash_cursor.set(cursor);
    hash_has_more.set(cursor != 0);
    hash_loading_more.set(false);
}

pub(super) async fn search_zset_server(
    pool: ConnectionPool,
    key: String,
    pattern: String,
    mut zset_value: Signal<Vec<(String, f64)>>,
    mut zset_cursor: Signal<u64>,
    mut zset_has_more: Signal<bool>,
    mut zset_loading_more: Signal<bool>,
) {
    zset_loading_more.set(true);
    let redis_pattern = if pattern.is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", pattern)
    };
    let mut cursor: u64 = 0;
    let mut all_items: Vec<(String, f64)> = Vec::new();
    let max_iterations = 1000;
    let mut iterations = 0;
    loop {
        match pool
            .zset_scan_match(&key, &redis_pattern, cursor, super::PAGE_SIZE)
            .await
        {
            Ok((new_cursor, items)) => {
                all_items.extend(items);
                cursor = new_cursor;
                iterations += 1;
                if cursor == 0 || iterations >= max_iterations {
                    break;
                }
                if !all_items.is_empty() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("搜索 zset 数据失败: {}", e);
                zset_loading_more.set(false);
                return;
            }
        }
    }
    zset_value.set(all_items);
    zset_cursor.set(cursor);
    zset_has_more.set(cursor != 0);
    zset_loading_more.set(false);
}

pub(super) async fn search_set_server(
    pool: ConnectionPool,
    key: String,
    pattern: String,
    mut set_value: Signal<Vec<String>>,
    mut set_cursor: Signal<u64>,
    mut set_has_more: Signal<bool>,
    mut set_loading_more: Signal<bool>,
) {
    set_loading_more.set(true);
    let redis_pattern = if pattern.is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", pattern)
    };
    let mut cursor: u64 = 0;
    let mut all_items: Vec<String> = Vec::new();
    let max_iterations = 1000;
    let mut iterations = 0;
    loop {
        match pool
            .set_scan_match(&key, &redis_pattern, cursor, super::PAGE_SIZE)
            .await
        {
            Ok((new_cursor, items)) => {
                all_items.extend(items);
                cursor = new_cursor;
                iterations += 1;
                if cursor == 0 || iterations >= max_iterations {
                    break;
                }
                if !all_items.is_empty() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("搜索 set 数据失败: {}", e);
                set_loading_more.set(false);
                return;
            }
        }
    }
    set_value.set(all_items);
    set_cursor.set(cursor);
    set_has_more.set(cursor != 0);
    set_loading_more.set(false);
}
