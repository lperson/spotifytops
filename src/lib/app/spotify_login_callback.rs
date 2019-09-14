use hyper::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Body, Request, Response,
};
extern crate tokio;
use futures::{prelude::*, Future};
use simple_error::SimpleError;
use serde_json;
use uuid::Uuid;

use super::super::app::STATE;
use super::super::server::helpers as server_helpers;
use super::super::spotify::auth::{token_request, token_response};
use super::super::CONFIG;
use super::super::server::ResponseFuture;


fn make_new_uuid() -> String {
    Uuid::new_v4()
        .to_hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_string()
}

pub fn handle(req: &Request<Body>) -> ResponseFuture {
    let parameters = server_helpers::get_query(&req).unwrap();
    let code = &parameters.get("code").unwrap().clone().unwrap();

    let token_request_payload = token_request::TokenRequest::get_serialized_request(&code);

    let request = Request::builder()
        .method("POST")
        .uri("https://accounts.spotify.com/api/token")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(CONTENT_LENGTH, token_request_payload.len())
        .body(Body::from(token_request_payload))
        .unwrap();

    let the_future = STATE
        .http_client
        .request(request)
        .map_err(|_| SimpleError::new("error with token request"))
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
                    server_helpers::redirect(
                        &mut response,
                        format!("{}/?t={}", CONFIG.redirect_host_and_port, uuid).as_str(),
                    );
                    Ok(response)
                })
                .map(|result| result.unwrap())
                .map_err(|_| SimpleError::new("error retrieving token response body"))
        })
        .map_err(|x| {
            println!("SHORT CIRCUITED! ==> {:?}", x);
            x
        });

    Box::new(the_future)
}
