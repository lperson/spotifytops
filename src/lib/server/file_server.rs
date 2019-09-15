use futures::future;
use futures::{prelude::*, Poll, Stream};
use hyper::{Body, Response};
use simple_error::SimpleError;
use tokio::fs::file::OpenFuture;
use tokio::io::AsyncRead;

use std::boxed::Box;
//use std::path::Path;

//use super::super::server::helpers::file_path_mime;
use super::super::types::ResponseFuture;
use super::super::CONFIG;

const CHUNK_SIZE: usize = 1024;

enum FileServerState {
    Opening,
    Reading,
}

pub struct FileServer {
    state: FileServerState,
    open_future: OpenFuture<String>,
    stream: Option<tokio::fs::File>,
    buf: [u8; CHUNK_SIZE],
}

// todo(LMP) need 404
impl FileServer {
    pub fn serve(root: &str, path: &str) -> ResponseFuture {
        let relative_path = &path[root.len()..];
        let open_future = tokio::fs::File::open(format!("{}/{}", CONFIG.static_dir, relative_path));
        let file_server = FileServer {
            state: FileServerState::Opening,
            open_future,
            stream: None,
            buf: [0; CHUNK_SIZE],
        };

        let response = Response::<Body>::new(Body::wrap_stream(file_server));
        Box::new(future::ok(response))
    }
}

impl Stream for FileServer {
    type Item = Vec<u8>;
    type Error = SimpleError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.state {
            FileServerState::Opening => match self.open_future.poll() {
                Ok(Async::Ready(file)) => {
                    self.state = FileServerState::Reading;
                    self.stream = Some(file);
                    self.poll()
                }

                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(e) => {
                    println!("Error opening file: {:?}", e);
                    Err(SimpleError::new("error opening file"))
                }
            },

            FileServerState::Reading => {
                let mut stream = None;
                std::mem::swap(&mut stream, &mut self.stream);
                let mut unwrapped_stream = stream.unwrap();
                let result = match unwrapped_stream.poll_read(&mut self.buf) {
                    Ok(Async::Ready(0)) => Ok(Async::Ready(None)),

                    Ok(Async::Ready(bytes_read)) => {
                        let mut value = vec![0; bytes_read];
                        value.as_mut_slice()[0..bytes_read]
                            .copy_from_slice(&self.buf[0..bytes_read]);
                        Ok(Async::Ready(Some(value)))
                    }

                    Ok(Async::NotReady) => Ok(Async::NotReady),

                    Err(_) => Err(SimpleError::new("error in FileServer")),
                };
                let mut stream = Some(unwrapped_stream);
                std::mem::swap(&mut stream, &mut self.stream);
                result
            }
        }
    }
}
