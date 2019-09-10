use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio::timer::Delay;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use libspotifytops::app::spotify_login_callback;
use libspotifytops::spotify::auth::*;
use libspotifytops::server::server;

use simple_error::SimpleError;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

fn make_handler(
    counter: Arc<Mutex<RefCell<u32>>>,
) -> Box<dyn FnMut(Request<Body>) -> BoxFut + Send> {
    Box::new(move |req: Request<Body>| -> BoxFut {
        let mut response = Response::new(Body::empty());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                //let mut counter_cell = counter.lock().unwrap();
                //let counter = counter_cell.get_mut();
                //*counter += 1;

                server::redirect(&mut response, &get_redirect(&req.uri().to_string()));
            }

            (&Method::GET, "/test/") => {
                println!("RECEIVED /test ==> {:?}", req);

                let when = Instant::now() + Duration::from_secs(3);

                let first = || {
                    println!("first");
                    future::ok(Response::<Body>::new(Body::empty()))
                };
                let second = || println!("second");
                let third = || println!("third");
                let fourth = || println!("fourth");
                let fifth = || println!("fifth");
                let sixth = || println!("sixth");

                let the_future = Delay::new(when)
                    .and_then(move |_| {
                        println!("delay reached");
                        future::ok(Response::<Body>::new(Body::empty()))
                    })
                    .join(first())
                    .and_then(move |_| first())
                    .and_then(|_| future::ok(Response::<Body>::new(Body::empty())))
                    .map_err(|e| panic!("delay errored; err={:?}", e));

                return Box::new(the_future);
            }

            (&Method::GET, "/SpotifyLoginCallback/") => {
                return spotify_login_callback::handle(&req);
            }

            _ => {
                println!("CATCHALL {} {:?}", req.uri().path(), req);
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        };

        Box::new(future::ok(response))
    })
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