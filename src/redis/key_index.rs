use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug)]
pub enum KeyIndexError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for KeyIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "key index I/O error: {error}"),
            Self::Sqlite(error) => write!(f, "key index database error: {error}"),
        }
    }
}

impl std::error::Error for KeyIndexError {}

impl From<std::io::Error> for KeyIndexError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for KeyIndexError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedKey {
    pub id: i64,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedTreeEntry {
    pub parent: Vec<u8>,
    pub name: Vec<u8>,
    pub path: Vec<u8>,
    pub is_leaf: bool,
    pub total_keys: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyQuery {
    pub contains: Option<Vec<u8>>,
    pub offset: u64,
    pub limit: usize,
}

impl Default for KeyQuery {
    fn default() -> Self {
        Self {
            contains: None,
            offset: 0,
            limit: 500,
        }
    }
}

/// A temporary per-scan index. It is removed on drop and never becomes app state.
pub struct KeyIndex {
    path: PathBuf,
    connection: Connection,
}

impl KeyIndex {
    pub fn new_temp() -> Result<Self, KeyIndexError> {
        let directory = std::env::temp_dir().join("rust-redis-desktop");
        std::fs::create_dir_all(&directory)?;
        let path = directory.join(format!("keys-{}.sqlite3", Uuid::new_v4()));
        let connection = Connection::open(&path)?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE keys (
                 id INTEGER PRIMARY KEY,
                 key BLOB NOT NULL UNIQUE
             );
             CREATE INDEX keys_lexical ON keys(key);
             CREATE TABLE tree_entries (
                 parent BLOB NOT NULL,
                 name BLOB NOT NULL,
                 path BLOB NOT NULL,
                 is_leaf INTEGER NOT NULL,
                 total_keys INTEGER NOT NULL DEFAULT 0,
                 PRIMARY KEY(parent, name, is_leaf)
             );
             CREATE INDEX tree_entries_parent ON tree_entries(parent, name, is_leaf);",
        )?;
        Ok(Self { path, connection })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn insert_batch(&mut self, keys: &[Vec<u8>]) -> Result<usize, KeyIndexError> {
        let tx = self.connection.transaction()?;
        let mut statement = tx.prepare("INSERT OR IGNORE INTO keys (key) VALUES (?1)")?;
        let mut tree_statement = tx.prepare(
            "INSERT INTO tree_entries
             (parent, name, path, is_leaf, total_keys) VALUES (?1, ?2, ?3, ?4, 1)
             ON CONFLICT(parent, name, is_leaf) DO UPDATE SET
             total_keys = total_keys + excluded.total_keys",
        )?;
        let mut inserted = 0;
        for key in keys {
            let changed = statement.execute(params![key])?;
            inserted += changed;
            if changed > 0 {
                insert_tree_entries(&mut tree_statement, key)?;
            }
        }
        drop(statement);
        drop(tree_statement);
        tx.commit()?;
        Ok(inserted)
    }

    pub fn count(&self) -> Result<u64, KeyIndexError> {
        Ok(self
            .connection
            .query_row("SELECT COUNT(*) FROM keys", [], |row| row.get::<_, i64>(0))?
            as u64)
    }

    pub fn page(&self, offset: u64, limit: usize) -> Result<Vec<IndexedKey>, KeyIndexError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, key FROM keys ORDER BY key, id LIMIT ?1 OFFSET ?2")?;
        let rows = statement.query_map(params![limit as i64, offset as i64], |row| {
            Ok(IndexedKey {
                id: row.get(0)?,
                key: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn search_page(&self, query: &KeyQuery) -> Result<Vec<IndexedKey>, KeyIndexError> {
        let Some(contains) = query.contains.as_ref() else {
            return self.page(query.offset, query.limit);
        };

        // instr over BLOB values preserves arbitrary Redis key bytes.
        let mut statement = self.connection.prepare(
            "SELECT id, key FROM keys
             WHERE instr(CAST(key AS BLOB), CAST(?1 AS BLOB)) > 0
             ORDER BY key, id LIMIT ?2 OFFSET ?3",
        )?;
        let rows = statement.query_map(
            params![contains, query.limit as i64, query.offset as i64],
            |row| {
                Ok(IndexedKey {
                    id: row.get(0)?,
                    key: row.get(1)?,
                })
            },
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn clear(&mut self) -> Result<(), KeyIndexError> {
        self.connection.execute("DELETE FROM keys", [])?;
        self.connection.execute("DELETE FROM tree_entries", [])?;
        Ok(())
    }

    pub fn tree_page(
        &self,
        parent: &[u8],
        offset: u64,
        limit: usize,
    ) -> Result<Vec<IndexedTreeEntry>, KeyIndexError> {
        let mut statement = self.connection.prepare(
            "SELECT parent, name, path, is_leaf, total_keys
             FROM tree_entries WHERE parent = ?1
             ORDER BY is_leaf, name LIMIT ?2 OFFSET ?3",
        )?;
        let rows = statement.query_map(params![parent, limit as i64, offset as i64], |row| {
            Ok(IndexedTreeEntry {
                parent: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                is_leaf: row.get::<_, i64>(3)? != 0,
                total_keys: row.get::<_, i64>(4)? as u64,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn tree_count(&self, parent: &[u8]) -> Result<u64, KeyIndexError> {
        Ok(self.connection.query_row(
            "SELECT COUNT(*) FROM tree_entries WHERE parent = ?1",
            params![parent],
            |row| row.get::<_, i64>(0),
        )? as u64)
    }

    pub fn contains(&self, key: &[u8]) -> Result<bool, KeyIndexError> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM keys WHERE key = ?1 LIMIT 1",
                params![key],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }
}

fn insert_tree_entries(
    statement: &mut rusqlite::Statement<'_>,
    key: &[u8],
) -> Result<(), KeyIndexError> {
    let mut parent = Vec::new();
    let mut start = 0;
    for (offset, byte) in key.iter().enumerate() {
        if *byte == b':' {
            let name = key[start..offset].to_vec();
            let path = key[..=offset].to_vec();
            statement.execute(params![&parent, &name, &path, 0i64])?;
            parent = path;
            start = offset + 1;
        }
    }
    statement.execute(params![&parent, &key[start..], key, 1i64])?;
    Ok(())
}

impl Drop for KeyIndex {
    fn drop(&mut self) {
        let _ = self
            .connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(format!("{}-wal", self.path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", self.path.display()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_deduplicates_and_pages_binary_keys() {
        let mut index = KeyIndex::new_temp().unwrap();
        let path = index.path().to_path_buf();
        assert_eq!(
            index
                .insert_batch(&[b"z".to_vec(), vec![0, 1, 2], b"a".to_vec(), b"z".to_vec()])
                .unwrap(),
            3
        );
        assert_eq!(index.count().unwrap(), 3);
        assert_eq!(
            index
                .page(0, 3)
                .unwrap()
                .into_iter()
                .map(|key| key.key)
                .collect::<Vec<_>>(),
            vec![vec![0, 1, 2], b"a".to_vec(), b"z".to_vec()]
        );
        assert!(index.contains(&[0, 1, 2]).unwrap());
        drop(index);
        assert!(!path.exists());
    }

    #[test]
    fn searches_literal_bytes_without_sql_wildcards() {
        let mut index = KeyIndex::new_temp().unwrap();
        index
            .insert_batch(&[b"user:%:1".to_vec(), b"user:2".to_vec(), b"other".to_vec()])
            .unwrap();
        let result = index
            .search_page(&KeyQuery {
                contains: Some(b"%:".to_vec()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, b"user:%:1");
    }

    #[test]
    fn clear_removes_all_rows() {
        let mut index = KeyIndex::new_temp().unwrap();
        index.insert_batch(&[b"one".to_vec()]).unwrap();
        index.clear().unwrap();
        assert_eq!(index.count().unwrap(), 0);
    }

    #[test]
    fn builds_paginated_tree_entries() {
        let mut index = KeyIndex::new_temp().unwrap();
        index
            .insert_batch(&[
                b"cache:user:1".to_vec(),
                b"cache:user:2".to_vec(),
                b"cache:status".to_vec(),
            ])
            .unwrap();
        let root = index.tree_page(b"", 0, 10).unwrap();
        assert_eq!(root.len(), 1);
        assert_eq!(root[0].name, b"cache");
        assert_eq!(root[0].total_keys, 3);
        let cache = index.tree_page(b"cache:", 0, 10).unwrap();
        assert_eq!(cache.len(), 2);
        assert_eq!(cache[0].name, b"user");
        assert_eq!(cache[1].name, b"status");
    }

    #[test]
    fn counts_and_pages_tree_entries_with_offsets() {
        let mut index = KeyIndex::new_temp().unwrap();
        index
            .insert_batch(&[b"a:1".to_vec(), b"b:1".to_vec(), b"c:1".to_vec()])
            .unwrap();

        assert_eq!(index.tree_count(b"").unwrap(), 3);
        assert_eq!(index.tree_page(b"", 1, 1).unwrap()[0].name, b"b");
        assert!(index.tree_page(b"", 3, 1).unwrap().is_empty());
    }
}
