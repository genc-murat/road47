use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct CacheEntry {
    pub value: Vec<u8>,
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
        let now = Instant::now();
        self.entries.get(key).and_then(|entry| {
            if entry.is_expired(now) {
                None
            } else {
                Some(&entry.value)
            }
        })
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) {
        let now = Instant::now();
        let entry = CacheEntry::new(value, self.ttl, now);
        self.entries.insert(key, entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_get_returns_none_when_entry_expired() {
        let mut cache = Cache::new(1); // TTL of 1 second
        cache.put("key".to_string(), vec![1, 2, 3]);

        // Wait for the entry to expire
        std::thread::sleep(Duration::from_secs(2));

        assert_eq!(cache.get("key"), None);
    }

    #[test]
    fn test_cache_get_returns_value_when_entry_not_expired() {
        let mut cache = Cache::new(10); // TTL of 10 seconds
        cache.put("key".to_string(), vec![1, 2, 3]);

        assert_eq!(cache.get("key"), Some(&vec![1, 2, 3]));
    }

    #[test]
    fn test_cache_put_overwrites_existing_entry() {
        let mut cache = Cache::new(10); // TTL of 10 seconds
        cache.put("key".to_string(), vec![1, 2, 3]);
        cache.put("key".to_string(), vec![4, 5, 6]);

        assert_eq!(cache.get("key"), Some(&vec![4, 5, 6]));
    }
}
