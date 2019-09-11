use hyper::{Body, Request, Response};

extern crate tokio;

use futures::{
    Future,
    prelude::*
};

use simple_error::SimpleError;

use serde_json;

use uuid::Uuid;

use super::super::server;
use super::super::spotify::auth::{token_request, token_response};
use super::super::app::STATE;
use super::super::CONFIG;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

fn make_new_uuid() -> String {
    Uuid::new_v4().to_hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_string()
}

pub fn handle(req: &Request<Body>) -> BoxFut {
    let parameters = server::get_query(&req).unwrap();
    let code = &parameters.get("code").unwrap().clone().unwrap();

    let token_request_payload = token_request::TokenRequest::get_serialized_request(&code);

    let request = Request::builder()
        .method("POST")
        .uri("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", token_request_payload.len())
        .body(Body::from(token_request_payload))
        .unwrap();

    let the_future = STATE.http_client
        .request(request)
        .map_err(|_| SimpleError::new("fuck you already"))
        .and_then(|result| {
            result
                .into_body()
                .concat2()
                .map(move |body| {
                    let token_response: token_response::TokenResponse = serde_json::from_str(
                        std::str::from_utf8(&body).unwrap(),
                    )
                    .unwrap_or_else(|e| {
                        println!("Error parsing token response {}", e);
                        token_response::TokenResponse::new_error(
                            "Error parsing token response".to_string(),
                            format!("{}", e),
                        )
                    });

                    if token_response.error.is_some() {
                        return Err(Box::new(SimpleError::new("parse error")));
                    }

                    let uuid = make_new_uuid();
                    let token = token_response.access_token.unwrap();

                    {
                        let mut tokens = STATE.tokens.lock().unwrap();
                        tokens.insert(uuid.clone(), token);
                    }

                    let mut response = Response::new(Body::empty());
                    server::redirect(&mut response, format!("{}/?t={}", CONFIG.redirect_host_and_port, uuid).as_str());
                    Ok(response)
                })
                .map(|result| result.unwrap() )
                .map_err(|_| SimpleError::new("c'mon"))
        })
        .map_err(|x| {
            println!("SHORT CIRCUITED! ==> {:?}", x);
            x
        });

    Box::new(the_future)
}
