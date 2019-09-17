use futures::future;
use futures::{prelude::*, Poll, Stream};
use hyper::{Body, Response as HyperResponse, StatusCode};
use simple_error::SimpleError;
use tokio::io::AsyncRead;

use std::boxed::Box;
//use std::path::Path;

//use super::super::server::helpers::file_path_mime;
use super::super::Response;
use super::super::types::ResponseFuture;
use super::super::CONFIG;

const CHUNK_SIZE: usize = 1024;

pub struct FileServer {
    file: tokio::fs::File,
    buf: [u8; CHUNK_SIZE],
}

impl FileServer {
    pub fn serve(root: &str, path: &str) -> ResponseFuture {
        let relative_path = &path[root.len()..];
        let the_future = tokio::fs::File::open(format!("{}/{}", CONFIG.static_dir, relative_path))
            .map(|file| {
                HyperResponse::<Body>::new(Body::wrap_stream(FileServer {
                    file,
                    buf: [0; CHUNK_SIZE],
                }))
            })
            .or_else(|e| {
                println!("ERROR OPENING FILE {:?}", e);
                let response: HyperResponse<Body> = Response::with_status(StatusCode::NOT_FOUND).into();
                future::ok(response)
            });

        Box::new(the_future)
    }
}

impl Stream for FileServer {
    type Item = Vec<u8>;
    type Error = SimpleError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.file.poll_read(&mut self.buf) {
            Ok(Async::Ready(0)) => Ok(Async::Ready(None)),

            Ok(Async::Ready(bytes_read)) => {
                let mut value = vec![0; bytes_read];
                value.as_mut_slice()[0..bytes_read].copy_from_slice(&self.buf[0..bytes_read]);
                Ok(Async::Ready(Some(value)))
            }

            Ok(Async::NotReady) => Ok(Async::NotReady),

            Err(e) => {
                println!("Error reading file: {:?}", e);
                Err(SimpleError::new("error in FileServer"))
            }
        }
    }
}
