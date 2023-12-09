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

#[cfg(test)]
mod tests {
    use super::*;

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

}
