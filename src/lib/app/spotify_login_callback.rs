pub mod spotify_login_callback {
    extern crate hyper;
    use hyper::{Body, Method, Request, Response, Server, StatusCode};

    extern crate futures;
    extern crate hyper_tls;

    extern crate tokio;

    use futures::{future::lazy, prelude::*, Async, Future};
    use hyper::Client;
    use hyper_tls::HttpsConnector;

    use super::super::super::spotify::auth::token_request;

    pub fn handle(request: &Request<Body>, response: &mut Response<Body>) {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let token_request_payload = token_request::TokenRequest::get_serialized_request(&"foo".to_string());

        let mut request = Request::builder()
            .method("POST")
            .uri("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Content-Length", token_request_payload.len())
            .body(Body::from(token_request_payload))
            .unwrap();

        hyper::rt::spawn(
            lazy(move || {
                client
                    .request(request)
                    .map(|result| {

                        hyper::rt::spawn(lazy(move || {


                                    result.into_body().concat2().map( |body|
                                        println!("GET RESULT {:?}", body)
                                    ).map_err(|e| eprintln!("body error: {}", e))
                                    .into_future()

                        }
                        ).map_err(|e| eprintln!("BODY SPAWN ERROR {:?}", e))
                        );
                    })
                    .map_err(|e| eprintln!("server error: {}", e))
                    .into_future()
            })
            .map_err(|e| eprintln!("outer error: {:?}", e)),
        );
    }
}
