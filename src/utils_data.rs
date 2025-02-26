use std::{
    collections::VecDeque,
    sync::{Arc, RwLock, RwLockReadGuard},
};

pub struct SlidingWindow<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T> SlidingWindow<T> {
    pub fn new (capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn add(&mut self, value: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn last_n(&self, item_count: usize) -> Result<impl Iterator<Item = &T>, String> {
        if item_count <= self.capacity {
            Ok(self.data.range(self.capacity - item_count..=self.capacity))
        } else {
            Err(format!("Utils SlidingWindow: Index {item_count} is out of bounds."))
        }
    }
}

#[derive(Debug)]
pub struct ReadOnlyRwLock<T>(Arc<RwLock<T>>);

impl<T> ReadOnlyRwLock<T> {
    pub fn from(init_arc: Arc<RwLock<T>>) -> Self {
        Self(init_arc)
    }

    pub fn read(&self) -> std::sync::LockResult<RwLockReadGuard<T>> {
        self.0.read()
    }
}
