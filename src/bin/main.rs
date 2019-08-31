extern crate futures;
extern crate hyper;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::http::{HeaderValue};
use hyper::header::{LOCATION};

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use libspotifytops::spotify::auth::*;
use libspotifytops::server::*;
use libspotifytops::app::*;
use libspotifytops::app::spotify_login_callback::spotify_login_callback;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn make_handler(counter: Arc<Mutex<RefCell<u32>>>) -> Box<dyn FnMut(Request<Body>) -> BoxFut + Send> {
    return Box::new(move |req: Request<Body>| -> BoxFut {
        let mut response = Response::new(Body::empty());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                //let mut counter_cell = counter.lock().unwrap();
                //let counter = counter_cell.get_mut();
                //*counter += 1;

                server::redirect(
                    &mut response,
                    &get_redirect(&req.uri().to_string())
                );
            }

            (&Method::GET, "/SpotifyLoginCallback/") => {
                println!("{:?}", req);
                spotify_login_callback::handle(&req, &mut response);
            }

            _ => {
                println!("CATCHALL {} {:?}", req.uri().path(), req);
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        };

        Box::new(future::ok(response))
    });
}

fn main() {

    let addr = ([127, 0, 0, 1], 8080).into();

    let counter_mutex = Arc::new(Mutex::new(RefCell::new(0)));

    let server = Server::bind(&addr)
        .serve(move || {
            let counter_mutex = Arc::clone(&counter_mutex);
            service_fn(make_handler(counter_mutex))
        })
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
