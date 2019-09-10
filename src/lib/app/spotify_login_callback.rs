use hyper::{Body, Request, Response};

extern crate tokio;

use futures::{
    future::{join_all},
    prelude::*,
    Future,
};

use simple_error::SimpleError;

use std::collections::btree_map::BTreeMap;

use serde_json;

use uuid::Uuid;

use super::super::server;
use super::super::spotify::auth::{token_request, token_response};

use super::super::spotify_future::SpotifyFuture;

use super::super::spotify::Retriever;
use super::super::spotify::TopArtistResponse;
use super::super::spotify::TopTrackResponse;
use super::super::app::STATE;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

fn make_new_uuid() -> String {
    Uuid::new_v4().to_hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_string()
}

pub fn handle(req: &Request<Body>) -> BoxFut {
    //println!("RECEIVED REQUEST ==> {:?}", req);


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

    let the_future = STATE.http_client
        .request(request)
        .map_err(|_| SimpleError::new("fuck you already"))
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
                .map_err(|_| SimpleError::new("c'mon"))
                .and_then(move |x| {
                    let auth_code = if let Ok(auth_string) = x {
                        auth_string
                    } else {
                        // should never get here because of short circuiting
                        String::new()
                    };

                    let the_artists_futures = vec![
                        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(&auth_code, "artists", "short_term")),
                        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(&auth_code, "artists", "medium_term")),
                        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(&auth_code, "artists", "long_term")),
                    ];

                    let the_tracks_futures = vec![
                        SpotifyFuture::<TopTrackResponse>::new(Retriever::new(&auth_code, "tracks", "short_term")),
                        SpotifyFuture::<TopTrackResponse>::new(Retriever::new(&auth_code, "tracks", "medium_term")),
                        SpotifyFuture::<TopTrackResponse>::new(Retriever::new(&auth_code, "tracks", "long_term")),
                    ];

                    join_all(the_artists_futures).join(join_all(the_tracks_futures))
                })
                .map(|(artists_results, tracks_results)| {
                    // for result in artists_results.iter().zip(tracks_results) {
                    //     println!("{:?}", result);
                    // }

                    let mut data = BTreeMap::new();
                    data.insert("test", serde_json::to_value("Hello Larry").unwrap());
                    data.insert("artists", serde_json::to_value(artists_results).unwrap());
                    data.insert("tracks", serde_json::to_value(tracks_results).unwrap()); 

                    let rendered = STATE.handlebars.render("tops", &data).unwrap();

                    println!("{}", rendered);

                    Response::<Body>::new(Body::empty())
                })
                .map_err(|x| x)
        })
        .map(|_| {
            Response::<Body>::new(Body::empty())
        })
        .map_err(|x| {
            println!("SHORT CIRCUITED! ==> {:?}", x);
            x
        });

    Box::new(the_future)
}
