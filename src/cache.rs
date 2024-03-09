use std::collections::{HashMap, VecDeque};
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
    entries: HashMap<String, CacheEntry>,
    keys: VecDeque<String>,
    ttl: Duration,
    capacity: usize,
}

impl Cache {
    pub fn new(ttl_seconds: u64, capacity: usize) -> Self {
        Cache {
            entries: HashMap::new(),
            keys: VecDeque::new(),
            ttl: Duration::from_secs(ttl_seconds),
            capacity,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&Vec<u8>> {
        let now = Instant::now();
        // Öncelikle, anahtarın süresi dolmuş mu diye kontrol et
        // Eğer dolmuşsa, hem entries'den hem de keys'den kaldır
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired(now) {
                self.entries.remove(key);
                self.keys.retain(|k| k != key);
                return None;
            }
        }

        // Eğer süresi dolmamışsa, anahtarı en son kullanılan olarak işaretle
        // ve değeri döndür
        if self.entries.contains_key(key) {
            self.mark_recently_used(key);
            // Bu noktada, güvenli bir şekilde değeri döndürebiliriz
            // Çünkü yukarıdaki satırlar mutable referansları zaten kullanıp bitirdi
            return self.entries.get(key).map(|e| &e.value);
        }
        None
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) {
        let now = Instant::now();
        let entry = CacheEntry::new(value, self.ttl, now);

        if self.entries.insert(key.clone(), entry).is_none() {
            // Yeni bir öğe eklendiğinde, LRU sırasını güncelle
            self.keys.push_front(key.clone());
            if self.keys.len() > self.capacity {
                if let Some(oldest) = self.keys.pop_back() {
                    self.entries.remove(&oldest);
                }
            }
        } else {
            // Mevcut bir öğe güncellendiğinde, LRU sırasını güncelle
            self.mark_recently_used(&key);
        }
    }

    // Anahtarı LRU listesinde en öne al
    fn mark_recently_used(&mut self, key: &str) {
        let index = self.keys.iter().position(|k| k == key).unwrap();
        let recent_key = self.keys.remove(index).unwrap();
        self.keys.push_front(recent_key);
    }
}
