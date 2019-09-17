use super::super::throttling_future::Throttle;

const MAX_CONCURRENT_EXECUTING_REQUESTS: usize = 4;

lazy_static! {
    pub static ref THROTTLE: Throttle = Throttle::new(MAX_CONCURRENT_EXECUTING_REQUESTS);
}