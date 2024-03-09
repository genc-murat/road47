use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct CacheEntry {
    value: Vec<u8>,
    expiry: Instant,
}

impl CacheEntry {
    pub fn new(value: Vec<u8>, ttl: Duration, now: Instant) -> Self {
        CacheEntry {
            value,
            expiry: now + ttl,
        }
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        now > self.expiry
    }
}

pub struct Cache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    keys: Mutex<VecDeque<String>>,
    ttl: Duration,
    capacity: usize,
}

impl Cache {
    pub fn new(ttl_seconds: u64, capacity: usize) -> Self {
        Cache {
            entries: Mutex::new(HashMap::new()),
            keys: Mutex::new(VecDeque::new()),
            ttl: Duration::from_secs(ttl_seconds),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let now = Instant::now();
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get(key) {
            if entry.is_expired(now) {
                entries.remove(key);
                self.keys.lock().unwrap().retain(|k| k != key);
                return None;
            }
        }

        if entries.contains_key(key) {
            self.mark_recently_used(key);
            return entries.get(key).map(|e| e.value.clone());
        }
        None
    }

    pub fn put(&self, key: String, value: Vec<u8>) {
        let now = Instant::now();
        let entry = CacheEntry::new(value, self.ttl, now);
        let mut entries = self.entries.lock().unwrap();

        if entries.insert(key.clone(), entry).is_none() {
            let mut keys = self.keys.lock().unwrap();
            keys.push_front(key.clone());
            if keys.len() > self.capacity {
                if let Some(oldest) = keys.pop_back() {
                    entries.remove(&oldest);
                }
            }
        } else {
            self.mark_recently_used(&key);
        }
    }

    fn mark_recently_used(&self, key: &str) {
        let mut keys = self.keys.lock().unwrap();
        if let Some(index) = keys.iter().position(|k| k == key) {
            let recent_key = keys.remove(index).unwrap();
            keys.push_front(recent_key);
        }
    }
}
