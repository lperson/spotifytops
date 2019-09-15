use futures::future::Future;
use hyper::{Response, Body};
use simple_error::SimpleError;

pub type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;