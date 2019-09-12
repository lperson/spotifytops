use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use libspotifytops::app::spotify_login_callback;
use libspotifytops::app::spotify_tops;
use libspotifytops::app::STATE;
use libspotifytops::server;
use libspotifytops::spotify::auth::*;
use libspotifytops::CONFIG;

use simple_error::SimpleError;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

fn make_handler() -> Box<dyn FnMut(Request<Body>) -> BoxFut + Send> {
    Box::new(move |req: Request<Body>| -> BoxFut {
        println!("{:?}", req);
        let mut response = Response::new(Body::empty());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                if let Some(query) = server::get_query(&req) {
                    if let Some(token) = query.get("t") {
                        if let Some(token) = token {
                            let mut auth_code: Option<String> = None;
                            {
                                let mut tokens = STATE.tokens.lock().unwrap();
                                if let Some(stored_auth_code) = tokens.get(&token) {
                                    auth_code = Some(stored_auth_code.clone());
                                }
                            }
                            if auth_code.is_some() {
                                return spotify_tops::handle(&auth_code.unwrap());
                            }
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
    let addr = format!("{}:{}", CONFIG.listen_addr, CONFIG.listen_port)
        .parse()
        .unwrap();

    let server = Server::bind(&addr)
        .serve(move || service_fn(make_handler()))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
