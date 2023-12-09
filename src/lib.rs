use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
use parking_lot::RwLock;
use time::OffsetDateTime;
pub use time::Duration;
pub use time::ext::NumericalDuration;

#[derive(Clone)]
struct Engram<T> (T, OffsetDateTime);

#[derive(Clone)]
pub struct Memory<T> {
    memory: Arc<RwLock<HashMap<String, Engram<T>>>>,
    retention: Duration,
}
impl<T> Memory<T> {
    pub fn new(retention: Duration) -> Self {
        Self {
            memory: Default::default(),
            retention,
        }
    }
}
impl<T: Clone> Memory<T> {
    pub fn memoize(&self, key: &str, value: T) {
        self.memory.write().insert(key.to_string(), Engram(value, OffsetDateTime::now_utc()));
    }
    pub fn forget(&self) {
        let now = OffsetDateTime::now_utc();
        let mut binding = self.memory.write();
        let vec = binding.iter().map(|(key, value)| (key.to_owned(), value.clone())).collect::<Vec<_>>();
        vec.iter().for_each(|(key, value)| {
            if value.1.add(self.retention) < now {
                let _ = binding.remove(key.as_str());
            }
        });
    }
    pub fn retrieve(&self, key: &str) -> Option<T> {
        self.memory.read().get(key).map(|engram| &engram.0).cloned()
    }
}
impl<T: Default + Clone> Memory<T> {
    pub fn retrieve_to_value(&self, key: &str) -> T {
        self.memory.read().get(key).map(|engram| &engram.0).cloned().unwrap_or(T::default())
    }
}

pub struct Alias<'map, 'memory, T> {
    map: &'map HashMap<String, String>,
    memory: &'memory Memory<T>,
}
impl<'map, 'memory, T> Alias<'map, 'memory, T> {
    pub fn new(memory: &'memory Memory<T>, map: &'map HashMap<String, String>) -> Self {
        Self { map, memory }
    }
}
impl<T: Clone> Alias<'_, '_, T> {
    pub fn memoize(&self, key: &str, value: T) {
        self.memory.memoize(self.map.get(key).unwrap_or(&key.to_string()), value);
    }
    pub fn retrieve(&self, key: &str) -> Option<T> {
        self.memory.retrieve(self.map.get(key).unwrap_or(&key.to_string()))
    }
}
impl<T: Default + Clone> Alias<'_, '_, T> {
    pub fn retrieve_to_value(&self, key: &str) -> T {
        self.memory.retrieve_to_value(self.map.get(key).unwrap_or(&key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_macros::hash_map;

    #[test]
    fn memory() {
        let memory = Memory::new(3.milliseconds());

        memory.memoize("a", 3);
        assert_eq!(memory.retrieve("a"), Some(3));
        assert_eq!(memory.retrieve_to_value("a"), 3);

        std::thread::sleep(std::time::Duration::from_millis(2));

        memory.memoize("b", 6);
        assert_eq!(memory.retrieve("b"), Some(6));
        assert_eq!(memory.retrieve_to_value("b"), 6);

        std::thread::sleep(std::time::Duration::from_millis(2));
        memory.forget();

        assert_eq!(memory.retrieve("a"), None);
        assert_eq!(memory.retrieve_to_value("a"), 0);
        assert_eq!(memory.retrieve("b"), Some(6));
    }

    #[test]
    fn alias() {
        let memory = Memory::new(3.milliseconds());

        memory.memoize("a", 3);
        memory.memoize("b", 6);

        let map = hash_map!(
            "aaa".to_string() => "a".to_string()
        );

        let alias = Alias::new(&memory, &map);

        assert_eq!(alias.retrieve("aaa"), Some(3));
        assert_eq!(alias.retrieve("bbb"), None);

        alias.memoize("aaa", 5);
        assert_eq!(alias.retrieve("aaa"), Some(5));
        assert_eq!(memory.retrieve("a"), Some(5));

        alias.memoize("ccc", 9);
        assert_eq!(alias.retrieve("ccc"), Some(9));
        assert_eq!(memory.retrieve("ccc"), Some(9));
    }

}
