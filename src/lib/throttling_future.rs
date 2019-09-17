use futures::{task, Async, Future, Poll};
use simple_error::SimpleError;
use std::boxed::Box;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

// https://tokio.rs/docs/internals/runtime-model/#yielding

const NUMBER_OF_CONCURRENT_FUTURES: usize = 1;

#[derive(Debug)]
pub struct ThrottlingFutureQueue {
    sequences_in_progress: Vec<usize>,
    waiting_sequences: VecDeque<usize>,
}

impl Default for ThrottlingFutureQueue {
    fn default() -> Self {
        Self {
            sequences_in_progress: Vec::with_capacity(NUMBER_OF_CONCURRENT_FUTURES),
            waiting_sequences: VecDeque::new(),
        }
    }
}

pub struct ThrottlingFutureThrottler {
    pub next_sequence: AtomicUsize,
    pub queue: RwLock<ThrottlingFutureQueue>,
}

impl Default for ThrottlingFutureThrottler {
    fn default() -> Self {
        Self {
            next_sequence: AtomicUsize::new(0),
            queue: RwLock::new(Default::default()),
        }
    }
}

lazy_static! {
    pub static ref THROTTLING_FUTURE_THROTTLER: ThrottlingFutureThrottler =
        Default::default();
}

pub struct ThrottlingFuture<I, E> {
    sequence: usize,
    processing: bool,
    future: Box<dyn Future<Item = I, Error = E> + Send>,
}

impl<I, E> ThrottlingFuture<I, E> {
    pub fn new(future: Box<dyn Future<Item = I, Error = E> + Send>) -> Self {
        println!("+++++++ +++++++ NEW");
        let sequence = THROTTLING_FUTURE_THROTTLER
            .next_sequence
            .fetch_add(1, Ordering::Relaxed);
        let return_value = Self {
            processing: false,
            sequence,
            future,
        };
        let mut writable_throttler_queue = THROTTLING_FUTURE_THROTTLER.queue.write().unwrap();
        writable_throttler_queue
            .waiting_sequences
            .push_back(sequence);
        return_value
    }
}

impl<I, E> Future for ThrottlingFuture<I, E> {
    type Item = I;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("+++++++ POLL {}", self.sequence);
        if !self.processing {
            {
                let readable_throttler_queue = THROTTLING_FUTURE_THROTTLER.queue.read().unwrap();
                println!("{:?}", readable_throttler_queue);

                let is_next = if let Some(front_sequence) = readable_throttler_queue.waiting_sequences.front() {
                    *front_sequence == self.sequence
                } else {
                    false
                };

                let number_in_progress = readable_throttler_queue.sequences_in_progress.len();
                if number_in_progress >= NUMBER_OF_CONCURRENT_FUTURES
                    || number_in_progress < NUMBER_OF_CONCURRENT_FUTURES && !is_next
                {
                    println!("______________ NOT READY TO PROCESS");
                    task::current().notify();
                    return Ok(Async::NotReady);
                }
            }

            {
                // TODO(lmp) get rid of code duplication
                let mut writable_throttler_queue =
                    THROTTLING_FUTURE_THROTTLER.queue.write().unwrap();
                let is_next = if let Some(front_sequence) = writable_throttler_queue.waiting_sequences.front() {
                    *front_sequence == self.sequence
                } else {
                    false
                };


                let number_in_progress = writable_throttler_queue.sequences_in_progress.len();
                if number_in_progress >= NUMBER_OF_CONCURRENT_FUTURES
                    || number_in_progress < NUMBER_OF_CONCURRENT_FUTURES && !is_next
                {
                    println!("______________ NOT READY TO PROCESS");
                    task::current().notify();
                    return Ok(Async::NotReady);
                }


                println!("******* HAPPINESS! READY TO PROCESS");
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
                    let mut writable_throttler_queue =
                        THROTTLING_FUTURE_THROTTLER.queue.write().unwrap();
                    let index_to_remove = writable_throttler_queue.sequences_in_progress.iter().position(
                        |&sequence| sequence == self.sequence
                    ).unwrap();
                    writable_throttler_queue.sequences_in_progress.remove(index_to_remove);
                }
                Ok(Async::Ready(t))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}
