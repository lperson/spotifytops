use futures::future;
use futures::future::lazy;
use futures::sync::oneshot;
use hyper::header::LOCATION;
use hyper::http::HeaderValue;
use hyper::rt::{Future, Stream, spawn};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde_json;
use tokio::timer::Delay;


use hyper::Client;
use hyper_tls::HttpsConnector;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::error::Error;

use libspotifytops::app::spotify_login_callback::spotify_login_callback;
use libspotifytops::app::*;
use libspotifytops::server::*;
use libspotifytops::spotify::auth::*;

use simple_error::SimpleError;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;


fn make_handler(
    counter: Arc<Mutex<RefCell<u32>>>,
) -> Box<dyn FnMut(Request<Body>) -> BoxFut + Send> {
    return Box::new(move |req: Request<Body>| -> BoxFut {
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
                    .and_then(
                        move |_| {
                            println!("delay reached");
                            future::ok(Response::<Body>::new(Body::empty()))
                        }
                    )
                    .join(first())
                    .and_then( move |_| first() )
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

/*
fn handle_spotify_login_callback(req: Request<Body>) -> BoxFut {
    println!("RECEIVED REQUEST ==> {:?}", req);
    //:write!return spotify_login_callback::handle(&req);

    let https = HttpsConnector::new(4).unwrap();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let parameters = server::get_query(&req).unwrap();
    let code = &parameters.get("code").unwrap().clone().unwrap();

    println!(
        "CODE ==> {:?}", code
    );

    let token_request_payload =
        token_request::TokenRequest::get_serialized_request(&code);

    let mut request = Request::builder()
        .method("POST")
        .uri("https://accounts.spotify.com/api/tokenx")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", token_request_payload.len())
        .body(Body::from(token_request_payload))
        .unwrap();

    println!("BODY ==> {:?}", request.body());

    // Result<Good, Err>
    // Ok(Good())
    // Err(SomeError())

    let the_future = client.request(request).and_then(move |result| {
        println!("STATUS_CODE ==> {}", result.status());
        result.into_body().concat2().map(move |body| {
            println!("RESPONSE ==> {:?}", body);
            let token_response: token_response::TokenResponse  =  
                serde_json::from_str(std::str::from_utf8(&body)
                .unwrap())
                .unwrap_or_else(|e|  {
                    Default::default()
                }
                );

            // let deser_result = serde_json::from_str(std::str::from_utf8(&body).unwrap());
            // let token_response = if let Ok(token_response) = deser_result {
            //     // do something with token response
            //     token_response
            // } else {
            //     let err = deser_result.unwrap_err();
            //     Default::default
            // }


            println!("TOKEN RESPONSE {:?}", token_response);

            println!("ACCESS TOKEN ==> {:?}", token_response.access_token);
            token_response.access_token.unwrap()
        })
        .and_then(move |x| {
            let request1 = Request::builder()
            .method("GET")
            .uri(format!("https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}","tracks","short_term"))
            .header("Authorization", format!("Bearer {}", x))
            .body(Body::empty())
            .unwrap();
            let future1 = client.request(request1)
                .and_then(
                    move |result| {
                        result.into_body().concat2().map(
                            move |body| {
                                body
                            }
                        )

                    }
            );

            let request2 = Request::builder()
            .method("GET")
            .uri(format!("https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}","tracks","short_term"))
            .header("Authorization", format!("Bearer {}", x))
            .body(Body::empty())
            .unwrap();
            let future2 = client.request(request2)
                .and_then(
                    move |result| {
                        result.into_body().concat2().map(
                            move |body| {
                                body
                            }
                        )

                    }
            );


            let request3 = Request::builder()
            .method("GET")
            .uri(format!("https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}","tracks","short_term"))
            .header("Authorization", format!("Bearer {}", x))
            .body(Body::empty())
            .unwrap();
            let future3 = client.request(request3)
                .and_then(
                    move |result| {
                        result.into_body().concat2().map(
                            move |body| {
                                body
                            }
                        )

                    }
            );

            future1.join(future2).join(future3)

        }).map(|((x1, x2), x3)| { 
                //println!("{:?}", x1);
                //println!("{:?}", x2);
                //println!("{:?}", x3);

                let response = Response::<Body>::new(Body::empty());
                response
            }
        )
        
    })
        .map(|x| { 
                let response = Response::<Body>::new(Body::empty());
                response
            }
        );

    return Box::new(the_future);
}
*/