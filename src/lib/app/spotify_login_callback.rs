use hyper::{Body, Request, Response};

extern crate tokio;

use futures::{
    future,
    future::{finished, join_all},
    prelude::*,
    Future,
};
use hyper::Client;
use hyper_tls::HttpsConnector;

use simple_error::SimpleError;

use super::super::server::*;
use super::super::spotify::auth::{token_request, token_response};

use super::super::spotify_future::SpotifyFuture;

use super::super::spotify::Retriever;
use super::super::spotify::TopArtistResponse;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

pub fn handle(req: &Request<Body>) -> BoxFut {
    //println!("RECEIVED REQUEST ==> {:?}", req);

    let https = HttpsConnector::new(4).unwrap();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let parameters = server::get_query(&req).unwrap();
    let code = &parameters.get("code").unwrap().clone().unwrap();

    // println!("CODE ==> {:?}", code);

    let token_request_payload = token_request::TokenRequest::get_serialized_request(&code);

    let request = Request::builder()
        .method("POST")
        .uri("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", token_request_payload.len())
        .body(Body::from(token_request_payload))
        .unwrap();

    println!("BODY ==> {:?}", request.body());

    let the_future = client
        .request(request)
        .map_err(|x| SimpleError::new("fuck you already"))
        .and_then(move |result| {
            println!("STATUS_CODE ==> {}", result.status());
            result
                .into_body()
                .concat2()
                .map(move |body| {
                    // println!("RESPONSE ==> {:?}", body);
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

                    //println!("TOKEN RESPONSE {:?}", token_response);

                    // println!("ACCESS TOKEN ==> {:?}", token_response.access_token);
                    Ok(token_response.access_token.unwrap())
                })
                .map_err(|x| SimpleError::new("c'mon"))
                .and_then(move |x| {
                    let auth_code = if let Ok(auth_string) = x {
                        auth_string
                    } else {
                        // should never get here because of short circuiting
                        String::new()
                    };

                    let the_futures = vec![
                        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(&auth_code, "artists", "short_term")),
                        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(&auth_code, "artists", "medium_term")),
                        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(&auth_code, "artists", "long_term")),
                        //SpotifyFuture::new(&auth_code, "artists", "medium_term"),
                        //SpotifyFuture::new(&auth_code, "artists", "long_term"),
                        //SpotifyFuture::new(&auth_code, "tracks", "short_term"),
                        //SpotifyFuture::new(&auth_code, "tracks", "medium_term"),
                        //SpotifyFuture::new(&auth_code, "tracks", "long_term"),
                    ];

                    join_all(the_futures)
                })
                .map(|results| {
                    for result in results {
                        println!("{:?}", result);
                    }

                    let response = Response::<Body>::new(Body::empty());
                    response
                })
                .map_err(|x| x)
        })
        .map(|x| {
            let response = Response::<Body>::new(Body::empty());
            response
        })
        .map_err(|x| {
            println!("SHORT CIRCUITED! ==> {:?}", x);
            x
        });

    Box::new(the_future)
}
