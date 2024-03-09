use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
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
    entries: RwLock<HashMap<String, CacheEntry>>,
    keys: RwLock<VecDeque<String>>,
    ttl: Duration,
    capacity: usize,
}

impl Cache {
    pub fn new(ttl_seconds: u64, capacity: usize) -> Self {
        Cache {
            entries: RwLock::new(HashMap::new()),
            keys: RwLock::new(VecDeque::new()),
            ttl: Duration::from_secs(ttl_seconds),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let now = Instant::now();
        {
            let entries = self.entries.read().unwrap();
            if let Some(entry) = entries.get(key) {
                if entry.is_expired(now) {
                    drop(entries); // Lock'u elden çıkar, böylece write lock alınabilir.
                    self.remove_entry(key);
                    return None;
                }
                return Some(entry.value.clone());
            }
        } // Okuma kilidinin erken serbest bırakılması için bu blok kullanılır.
        None
    }

    pub fn put(&self, key: String, value: Vec<u8>) {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();

        if entries
            .insert(key.clone(), CacheEntry::new(value, self.ttl, now))
            .is_none()
        {
            let mut keys = self.keys.write().unwrap();
            keys.push_front(key.clone());
            if keys.len() > self.capacity {
                if let Some(oldest) = keys.pop_back() {
                    entries.remove(&oldest);
                }
            }
        } else {
            drop(entries); // Lock'u elden çıkarıp, mark_recently_used içinde yeniden alınmasını sağlayabiliriz.
            self.mark_recently_used(&key);
        }
    }

    fn mark_recently_used(&self, key: &str) {
        let mut keys = self.keys.write().unwrap();
        if let Some(index) = keys.iter().position(|k| k == key) {
            let recent_key = keys.remove(index).unwrap();
            keys.push_front(recent_key);
        }
    }

    fn remove_entry(&self, key: &str) {
        let mut entries = self.entries.write().unwrap();
        entries.remove(key);
        let mut keys = self.keys.write().unwrap();
        keys.retain(|k| k != key);
    }
}
