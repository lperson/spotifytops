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

const CHUNK_SIZE: usize = 128;

pub enum FileServer {
    None,
    Opening(OpenFuture<String>),
    Reading((tokio::fs::File, [u8; CHUNK_SIZE])),
}

// todo(LMP) need 404
impl FileServer {
    pub fn serve(root: &str, path: &str) -> ResponseFuture {
        let relative_path = &path[root.len()..];
        let open_future = tokio::fs::File::open(format!("{}/{}", CONFIG.static_dir, relative_path));
        let file_server = FileServer::Opening(open_future);

        let response = Response::<Body>::new(Body::wrap_stream(file_server));
        Box::new(future::ok(response))
    }
}

impl Stream for FileServer {
    type Item = Vec<u8>;
    type Error = SimpleError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut temp_self = FileServer::None;
        std::mem::swap(&mut temp_self, self);

        match temp_self {
            FileServer::Opening(mut open_future) => match open_future.poll() {
                Ok(Async::Ready(file)) => {
                    let mut reading = FileServer::Reading((file, [0; CHUNK_SIZE]));
                    std::mem::swap(&mut reading, self);
                    self.poll()
                }

                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(e) => {
                    println!("Error opening file: {:?}", e);
                    Err(SimpleError::new("error opening file"))
                }
            },

            FileServer::Reading((mut file, mut buf)) => {
                let result = match file.poll_read(&mut buf) {
                    Ok(Async::Ready(0)) => Ok(Async::Ready(None)),

                    Ok(Async::Ready(bytes_read)) => {
                        let mut value = vec![0; bytes_read];
                        value.as_mut_slice()[0..bytes_read]
                            .copy_from_slice(&buf[0..bytes_read]);
                        Ok(Async::Ready(Some(value)))
                    }

                    Ok(Async::NotReady) => Ok(Async::NotReady),

                    Err(_) => Err(SimpleError::new("error in FileServer")),
                };

                let mut new_reading = FileServer::Reading((file, [0; CHUNK_SIZE]));
                std::mem::swap(&mut new_reading, self);
                result
            }

            FileServer::None => { 
                println!("Impossible condition achieved");
                Err(SimpleError::new("impossible condition achieved"))
            }
        }
    }
}
