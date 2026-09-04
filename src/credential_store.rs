use std::collections::HashMap;
use std::io;
use std::sync::Mutex;

const SERVICE: &str = "rust-redis-desktop";

pub trait CredentialStore: Send + Sync {
    fn put(&self, id: &str, secret: &str) -> io::Result<()>;
    fn get(&self, id: &str) -> io::Result<Option<String>>;
    fn delete(&self, id: &str) -> io::Result<()>;
}

#[derive(Clone, Copy)]
pub struct OsCredentialStore;

impl CredentialStore for OsCredentialStore {
    fn put(&self, id: &str, secret: &str) -> io::Result<()> {
        keyring::Entry::new(SERVICE, id)
            .and_then(|entry| entry.set_password(secret))
            .map_err(keyring_error)
    }

    fn get(&self, id: &str) -> io::Result<Option<String>> {
        match keyring::Entry::new(SERVICE, id).and_then(|entry| entry.get_password()) {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(keyring_error(error)),
        }
    }

    fn delete(&self, id: &str) -> io::Result<()> {
        match keyring::Entry::new(SERVICE, id).and_then(|entry| entry.delete_credential()) {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(keyring_error(error)),
        }
    }
}

fn keyring_error(error: keyring::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        format!("credential store error: {error}"),
    )
}

#[derive(Default)]
pub struct MemoryCredentialStore {
    secrets: Mutex<HashMap<String, String>>,
}

impl CredentialStore for MemoryCredentialStore {
    fn put(&self, id: &str, secret: &str) -> io::Result<()> {
        self.secrets
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "credential store lock poisoned"))?
            .insert(id.to_string(), secret.to_string());
        Ok(())
    }

    fn get(&self, id: &str) -> io::Result<Option<String>> {
        Ok(self
            .secrets
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "credential store lock poisoned"))?
            .get(id)
            .cloned())
    }

    fn delete(&self, id: &str) -> io::Result<()> {
        self.secrets
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "credential store lock poisoned"))?
            .remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip_and_delete() {
        let store = MemoryCredentialStore::default();
        store.put("id", "secret").unwrap();
        assert_eq!(store.get("id").unwrap().as_deref(), Some("secret"));
        store.delete("id").unwrap();
        assert_eq!(store.get("id").unwrap(), None);
    }
}
