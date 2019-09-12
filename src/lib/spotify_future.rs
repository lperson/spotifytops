use futures::{
    future,
    prelude::*,
    Future, Poll,
};
use hyper::{Chunk};
use hyper::{Body, Request};
use simple_error::SimpleError;

use super::app::STATE;

use super::spotify::top_items_retriever::{Retriever, Retrievable};
use serde::de::DeserializeOwned;

type BoxFut<T> = Box<dyn Future<Item = T, Error = SimpleError> + Send>;

pub struct SpotifyFuture<'a, T> {
    retriever: Retriever<T>,
    the_future: BoxFut<T>,
    count: u8,
    phantom: std::marker::PhantomData<&'a T>
}

impl<'a, T> SpotifyFuture<'a, T> 
        where T: DeserializeOwned
{
    fn make_future(retriever: &Retriever<T>) -> BoxFut<T>
        where T: DeserializeOwned
    {
        let request = Request::builder()
            .method("GET")
            .uri(retriever.uri.clone())
            .header("Authorization", retriever.authorization.clone())
            .body(Body::empty())
            .unwrap();

        let future = STATE.http_client
            .request(request)
            .map_err(|_| SimpleError::new("error getting data"))
            .and_then(move |result| {
                println!("{}", result.status());
                if [404u16, 400u16]
                    .iter()
                    .any(|x| result.status().as_u16() == *x)
                {
                    let this_return = future::err(SimpleError::new("400"));
                    return this_return;
                }

                future::ok(result)
            })
            .and_then(|result| {
                result
                    .into_body()
                    .concat2()
                    .map(
                        |chunk: Chunk| -> T
                    {
                        let body_vec = &chunk.to_vec();
                        let response_text = std::str::from_utf8(body_vec).unwrap();
                        Retriever::deserialize(
                            response_text
                        ).unwrap()
                    } 
                    )
                    .map_err(|_| SimpleError::new("error retrieving data"))
            });
        Box::new(future)
    }

    pub fn new(retriever: Retriever<T>) -> Self {
        let the_future = Self::make_future(&retriever);

        SpotifyFuture {
            retriever,
            the_future,
            count: 0,
            phantom: std::marker::PhantomData
        }
    }
}

impl<'a, T> Future for SpotifyFuture<'a, T> 
    where T: DeserializeOwned
{
    type Item = T;
    type Error = SimpleError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.the_future.poll() {
            Ok(Async::Ready(t)) => Ok(Async::Ready(t)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => {
                println!("IN POLL ==> {:?}", e);
                if e.as_str() == "400" && self.count == 1 {
                    println!("400 in poll");
                    return Err(e);
                }

                self.the_future = SpotifyFuture::make_future(&self.retriever);
                self.count = 1;
                self.the_future.poll()
            }
        }
    }
}
