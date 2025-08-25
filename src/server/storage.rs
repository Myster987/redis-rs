use std::{collections::HashMap, hash::Hash, sync::Mutex};

use crate::utils::command::Value;

pub struct Database<K: Hash + Eq> {
    map: Mutex<HashMap<K, Value>>,
}

impl<'a, K: Hash + Eq> Database<K> {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<Value> {
        let lock = self.map.lock().unwrap();
        lock.get(key).cloned()
    }

    pub fn set(&self, key: K, value: Value) -> Option<Value> {
        let mut lock = self.map.lock().unwrap();
        lock.insert(key, value)
    }

    pub fn delete(&self, key: &K) -> Option<Value> {
        let mut lock = self.map.lock().unwrap();
        lock.remove(key)
    }
}
