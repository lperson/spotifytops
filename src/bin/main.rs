use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response as HyperResponse, Server, StatusCode};

use libspotifytops::app::spotify_login_callback;
use libspotifytops::app::spotify_tops;
use libspotifytops::app::STATE;
use libspotifytops::server::{helpers as server_helpers, FileServer, ResponseFuture};
use libspotifytops::spotify::auth::*;
use libspotifytops::CONFIG;
use libspotifytops::server::Response;

fn make_handler() -> Box<dyn FnMut(Request<Body>) -> ResponseFuture + Send> {
    Box::new(move |req: Request<Body>| -> ResponseFuture {
        println!("{:?}", req);

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                if let Some(query) = server_helpers::get_query(&req) {
                    if let Some(token) = query.get("t") {
                        if let Some(token) = token {
                            let mut auth_code: Option<String> = None;
                            {
                                let tokens = STATE.tokens.read().unwrap();
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

                let response = Response::with_redirect(&get_redirect(&req.uri().to_string()));
                Box::new(future::ok(response.into()))
            }

            (&Method::GET, "/SpotifyLoginCallback/") => {
                spotify_login_callback::handle(&req)
            }

            (&Method::GET, path) if path.starts_with("/static/") => {
                // TODO(lmp) cache small files?
                FileServer::serve("/static/", path)
            }

            _ => {
                println!("CATCHALL {} {:?}", req.uri().path(), req);

                let response: HyperResponse<Body> = Response::with_status(StatusCode::NOT_FOUND).into();
                Box::new(future::ok(response))
            }
        }
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
