use std::sync::atomic::AtomicUsize;

pub struct RoughlySizedLockFreeStack<T> {
    size: AtomicUsize,
    max_size: usize,
    stack: lockfree::stack::Stack<T>,
}

pub enum RoughlySizedLockFreeStackPushResult {
    Done,
    Full,
}

impl<T> RoughlySizedLockFreeStack<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            size: AtomicUsize::new(0),
            stack: lockfree::stack::Stack::new(),
        }
    }

    pub fn pop(&self) -> Option<T> {
        let top = self.stack.pop();
        if top.is_some() {
            self.size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
        top
    }

    pub fn clear(&self) {
        while self.pop().is_none() {}
        self.size.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn push(&self, item: T) -> RoughlySizedLockFreeStackPushResult {
        if self.size.load(std::sync::atomic::Ordering::Relaxed) > self.max_size {
            return RoughlySizedLockFreeStackPushResult::Full;
        }
        self.stack.push(item);
        self.size.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        RoughlySizedLockFreeStackPushResult::Done
    }
}
