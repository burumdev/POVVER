use std::collections::VecDeque;

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

    pub fn add(&mut self, value: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn last_n(&self, n: usize) -> Result<impl Iterator<Item = &T>, String> {
        if n <= self.capacity {
            Ok(self.data.range(self.capacity - n..=self.capacity))
        } else {
            Err(format!("Utils SlidingWindow: Index {n} is out of bounds."))
        }
    }
}