use parking_lot::RwLock;
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
pub use time::ext::NumericalDuration;
pub use time::Duration;
use time::OffsetDateTime;

pub trait Memory<T> {
    fn memoize(&self, key: &str, value: T);
    fn retrieve(&self, key: &str) -> Option<T>;
    fn forget(&self);
}
pub trait MemoryDefaultRetrieval<T>: Memory<T> {
    fn retrieve_or_default(&self, key: &str) -> T;
}

#[derive(Clone)]
struct Engram<T>(T, OffsetDateTime);

#[derive(Clone)]
pub struct Brain<T> {
    memory: Arc<RwLock<HashMap<String, Engram<T>>>>,
    retention: Duration,
}
impl<T> Brain<T> {
    pub fn new(retention: Duration) -> Self {
        Self {
            memory: Default::default(),
            retention,
        }
    }
}
impl<T: Clone> Memory<T> for Brain<T> {
    fn memoize(&self, key: &str, value: T) {
        self.memory
            .write()
            .insert(key.to_string(), Engram(value, OffsetDateTime::now_utc()));
    }
    fn forget(&self) {
        let now = OffsetDateTime::now_utc();
        let mut binding = self.memory.write();
        let vec = binding
            .iter()
            .map(|(key, value)| (key.to_owned(), value.clone()))
            .collect::<Vec<_>>();
        vec.iter().for_each(|(key, value)| {
            if value.1.add(self.retention) < now {
                let _ = binding.remove(key.as_str());
            }
        });
    }
    fn retrieve(&self, key: &str) -> Option<T> {
        self.memory.read().get(key).map(|engram| &engram.0).cloned()
    }
}
impl<T: Default + Clone> MemoryDefaultRetrieval<T> for Brain<T> {
    fn retrieve_or_default(&self, key: &str) -> T {
        self.retrieve(key).unwrap_or(T::default())
    }
}

pub struct MemorySubstitute<'map, 'memory, T> {
    map: &'map HashMap<String, String>,
    memory: &'memory Brain<T>,
}
impl<'map, 'memory, T> MemorySubstitute<'map, 'memory, T> {
    pub fn new(memory: &'memory Brain<T>, map: &'map HashMap<String, String>) -> Self {
        Self { map, memory }
    }
}
impl<T: Clone> Memory<T> for MemorySubstitute<'_, '_, T> {
    fn memoize(&self, key: &str, value: T) {
        self.memory
            .memoize(self.map.get(key).unwrap_or(&key.to_string()), value);
    }
    fn retrieve(&self, key: &str) -> Option<T> {
        self.memory
            .retrieve(self.map.get(key).unwrap_or(&key.to_string()))
    }
    fn forget(&self) {
        self.memory.forget();
    }
}
impl<T: Default + Clone> MemoryDefaultRetrieval<T> for MemorySubstitute<'_, '_, T> {
    fn retrieve_or_default(&self, key: &str) -> T {
        self.memory
            .retrieve_or_default(self.map.get(key).unwrap_or(&key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_macros::hash_map;

    #[test]
    fn memory() {
        let memory = Brain::new(3.milliseconds());

        memory.memoize("a", 3);
        assert_eq!(memory.retrieve("a"), Some(3));
        assert_eq!(memory.retrieve_or_default("a"), 3);

        std::thread::sleep(std::time::Duration::from_millis(2));

        memory.memoize("b", 6);
        assert_eq!(memory.retrieve("b"), Some(6));
        assert_eq!(memory.retrieve_or_default("b"), 6);

        std::thread::sleep(std::time::Duration::from_millis(2));
        memory.forget();

        assert_eq!(memory.retrieve("a"), None);
        assert_eq!(memory.retrieve_or_default("a"), 0);
        assert_eq!(memory.retrieve("b"), Some(6));
    }

    #[test]
    fn alias() {
        let memory = Brain::new(3.milliseconds());

        memory.memoize("a", 3);
        memory.memoize("b", 6);

        let map = hash_map!(
            "aaa".to_string() => "a".to_string()
        );

        let alias = MemorySubstitute::new(&memory, &map);

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
