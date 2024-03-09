use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct CacheEntry {
    value: Vec<u8>,
    expiry: Instant,
    last_accessed: Instant,
}

impl CacheEntry {
    pub fn new(value: Vec<u8>, ttl: Duration, now: Instant) -> Self {
        CacheEntry {
            value,
            expiry: now + ttl,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        now > self.expiry
    }

    pub fn update_last_accessed(&mut self, now: Instant) {
        self.last_accessed = now;
    }
}

pub struct Cache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    ttl: Duration,
    capacity: usize,
}

impl Cache {
    pub fn new(ttl_seconds: u64, capacity: usize) -> Self {
        Cache {
            entries: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired(now) {
                entries.remove(key);
                return None;
            }
            entry.update_last_accessed(now);
            return Some(entry.value.clone());
        }
        None
    }

    pub fn put(&self, key: String, value: Vec<u8>) {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();

        let remove_key = if entries.len() == self.capacity && !entries.contains_key(&key) {
            entries
                .iter()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(k, _)| k.clone())
        } else {
            None
        };

        if let Some(k) = remove_key {
            entries.remove(&k);
        }

        entries.insert(key, CacheEntry::new(value, self.ttl, now));
    }
}
