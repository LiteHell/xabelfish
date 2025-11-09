use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

pub struct RoughlySizeConstraintDeque<T>
where
    T: Clone,
{
    max_size: usize,
    stack: Arc<RwLock<VecDeque<T>>>,
}

impl<T> RoughlySizeConstraintDeque<T>
where
    T: Clone,
{
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            stack: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub fn last(&self) -> Option<T> {
        let read_guard = self.stack.read().unwrap();
        let back = read_guard.back();
        if let Some(back) = back {
            Some(back.clone())
        } else {
            None
        }
    }

    pub fn clear(&self) {
        let mut stack = self.stack.write().unwrap();
        stack.clear();
    }

    pub fn push(&self, item: T) {
        let mut stack = self.stack.write().unwrap();
        while stack.len() >= self.max_size {
            stack.pop_front();
        }
        stack.push_back(item);
    }
}
