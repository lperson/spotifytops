use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use libspotifytops::app::spotify_login_callback;
use libspotifytops::app::spotify_tops;
use libspotifytops::spotify::auth::*;
use libspotifytops::server;

use simple_error::SimpleError;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

fn make_handler() -> Box<dyn FnMut(Request<Body>) -> BoxFut + Send> {
    Box::new(move |req: Request<Body>| -> BoxFut {
        let mut response = Response::new(Body::empty());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                if let Some(query) = server::get_query(&req) {
                    if let Some(token) = query.get("t") {
                        if let Some(token) = token {
                            return spotify_tops::handle(&req);
                        }
                    }
                } 
                
                server::redirect(&mut response, &get_redirect(&req.uri().to_string()));
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

    let server = Server::bind(&addr)
        .serve(move || {
            service_fn(make_handler())
        })
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}