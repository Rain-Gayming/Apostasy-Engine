use std::sync::mpsc::{self, Receiver, Sender};

use threadpool::ThreadPool;

use crate::game::world::chunk::Chunk;

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
    pub chunk_meshing_couple: ThreadCouple<Chunk>,
}

impl Default for ThreadManager {
    fn default() -> Self {
        ThreadManager {
            thread_pool: ThreadPool::new(4),
            chunk_meshing_couple: ThreadCouple::default(),
        }
    }
}
