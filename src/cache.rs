use std::collections::HashMap;

use std::time::{Duration, Instant};

pub struct CacheEntry {
    pub value: Vec<u8>,
    expiry: Instant,
}

impl CacheEntry {
    pub fn new(value: Vec<u8>, ttl: Duration) -> Self {
        CacheEntry {
            value,
            expiry: Instant::now() + ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expiry
    }
}

pub struct Cache {
    entries: HashMap<String, CacheEntry>,
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl_seconds: u64) -> Self {
        Cache {
            entries: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.entries.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(&entry.value)
            }
        })
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) {
        let entry = CacheEntry::new(value, self.ttl);
        self.entries.insert(key, entry);
    }
}
