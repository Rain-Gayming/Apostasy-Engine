use std::sync::mpsc::{self, Receiver, Sender};

use threadpool::ThreadPool;

pub struct ThreadCouple<T> {
    pub sender: Sender<T>,
    pub reciever: Receiver<T>,
}

impl<T> Default for ThreadCouple<T> {
    fn default() -> Self {
        let (sender, reciever) = mpsc::channel();
        ThreadCouple { sender, reciever }
    }
}

pub struct ThreadManager {
    pub thread_pool: ThreadPool,
}

impl Default for ThreadManager {
    fn default() -> Self {
        ThreadManager {
            thread_pool: ThreadPool::new(4),
        }
    }
}
