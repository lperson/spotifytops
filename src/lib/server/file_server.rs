use futures::future;
use futures::{prelude::*, Poll, Stream};
use hyper::{Body, Response};
use simple_error::SimpleError;
use tokio::io::AsyncRead;

use std::boxed::Box;

use super::super::types::ResponseFuture;
use super::super::CONFIG;
use super::super::server::helpers::file_path_mime;

const CHUNK_SIZE: usize = 1024;

pub struct FileServer {
    stream: tokio::fs::File,
    buf: [u8; CHUNK_SIZE],
}

// todo(LMP) need 404
impl FileServer {
    pub fn serve(root: &str, path: &str) -> ResponseFuture {
        let relative_path = &path[root.len()..];
        let stream = tokio::fs::File::from_std(
            std::fs::File::open(format!("{}/{}", CONFIG.static_dir, relative_path)).unwrap(),
        );
        let buf = [0; CHUNK_SIZE];
        let file_server = Self { stream, buf };

        let response = Response::<Body>::new(Body::wrap_stream(file_server));
        Box::new(future::ok(response))
    }
}

impl Stream for FileServer {
    type Item = Vec<u8>;
    type Error = SimpleError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.stream.poll_read(&mut self.buf) {
            Ok(Async::Ready(0)) => Ok(Async::Ready(None)),

            Ok(Async::Ready(bytes_read)) => {
                let mut value = vec![0; bytes_read];
                value.as_mut_slice()[0..bytes_read].copy_from_slice(&self.buf[0..bytes_read]);
                Ok(Async::Ready(Some(value)))
            }

            Ok(Async::NotReady) => Ok(Async::NotReady),

            Err(_) => Err(SimpleError::new("error in FileServer")),
        }
    }
}
