use futures::{task, Async, Future, Poll};
use std::boxed::Box;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    RwLock,
};

#[derive(Debug)]
pub struct ThrottlingFutureQueue {
    sequences_in_progress: Vec<usize>,
    waiting_sequences: VecDeque<usize>,
}

impl ThrottlingFutureQueue {
    pub(in self) fn new(max_concurrent_execution: usize) -> Self {
        Self {
            sequences_in_progress: Vec::with_capacity(max_concurrent_execution),
            waiting_sequences: VecDeque::new(),
        }
    }
}

pub struct Throttle {
    pub next_sequence: AtomicUsize,
    pub queue: RwLock<ThrottlingFutureQueue>,
    pub(in self) max_concurrent_execution: usize,
}

impl Throttle {
    pub fn new(max_concurrent_execution: usize) -> Self {
        Self {
            next_sequence: AtomicUsize::new(0),
            queue: RwLock::new(ThrottlingFutureQueue::new(max_concurrent_execution)),
            max_concurrent_execution,
        }
    }
}

pub struct ThrottlingFuture<I, E> {
    sequence: usize,
    processing: bool,
    future: Box<dyn Future<Item = I, Error = E> + Send>,
    throttle: &'static Throttle,
}

impl<I, E> ThrottlingFuture<I, E> {
    pub fn new(
        future: Box<dyn Future<Item = I, Error = E> + Send>,
        throttle: &'static Throttle,
    ) -> Self {
        let sequence = throttle.next_sequence.fetch_add(1, Ordering::Relaxed);
        let return_value = Self {
            processing: false,
            sequence,
            future,
            throttle,
        };
        let mut writable_throttler_queue = throttle.queue.write().unwrap();
        writable_throttler_queue
            .waiting_sequences
            .push_back(sequence);
        return_value
    }
    fn ready_to_start_polling(&self, queue: &ThrottlingFutureQueue) -> bool {
        let is_next = if let Some(front_sequence) = queue.waiting_sequences.front() {
            *front_sequence == self.sequence
        } else {
            false
        };

        let number_in_progress = queue.sequences_in_progress.len();
        number_in_progress < self.throttle.max_concurrent_execution && is_next
    }
}

impl<I, E> Future for ThrottlingFuture<I, E> {
    type Item = I;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if !self.processing {
            {
                let readable_throttler_queue = self.throttle.queue.read().unwrap();
                if !self.ready_to_start_polling(&*readable_throttler_queue) {
                    task::current().notify();
                    return Ok(Async::NotReady);
                }
            }

            {
                let mut writable_throttler_queue = self.throttle.queue.write().unwrap();
                if !self.ready_to_start_polling(&*writable_throttler_queue) {
                    task::current().notify();
                    return Ok(Async::NotReady);
                }

                writable_throttler_queue
                    .sequences_in_progress
                    .push(self.sequence);

                writable_throttler_queue.waiting_sequences.pop_front();
                self.processing = true;
            }
        }

        match self.future.poll() {
            Ok(Async::Ready(t)) => {
                {
                    let mut writable_throttler_queue = self.throttle.queue.write().unwrap();
                    let index_to_remove = writable_throttler_queue
                        .sequences_in_progress
                        .iter()
                        .position(|&sequence| sequence == self.sequence)
                        .unwrap();
                    writable_throttler_queue
                        .sequences_in_progress
                        .remove(index_to_remove);
                }
                Ok(Async::Ready(t))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}
