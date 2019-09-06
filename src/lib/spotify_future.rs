pub mod spotify_future {

    use futures::{future, future::finished, prelude::*, Future, Poll};
    use hyper::Client;
    use hyper::{Body, Request, Response};
    use hyper_tls::HttpsConnector;
    use simple_error::SimpleError;

    type BoxFut = Box<dyn Future<Item = String, Error = SimpleError> + Send>;
    
    pub struct SpotifyFuture {
        uri: String,
        authorization: String,
        the_future: BoxFut,
    }

    impl SpotifyFuture {
        pub fn new(auth_code: &String, what_to_get: &str, timeframe: &str) -> Self {
            let https = HttpsConnector::new(4).unwrap();
            let client = Client::builder().build::<_, hyper::Body>(https);

            let uri = format!(
                    "https://api.spotify.com/v1/me/topxx/{}?limit=50&time_range={}",
                    what_to_get, timeframe
                );

            let authorization = format!("Bearer {}", auth_code);

            let request = Request::builder()
                .method("GET")
                .uri(uri.clone())
                .header("Authorization", authorization.clone())
                .body(Body::empty())
                .unwrap();

            let future = client
                .request(request)
                .map_err(|x| future::err(SimpleError::new("x")))
                .and_then(move |result| {
                    println!("{}", result.status());
                    if result.status().as_u16() == 404 {
                        return future::err(SimpleError::new("parse error"));
                    }

                    result
                        .into_body()
                        .concat2()
                        .map(|x| future::ok(String::from_utf8(x.to_vec())))
                        .map_err(|x| future::err(SimpleError::new("x")))
                })
                //.map(|x| x)
                .map_err(|x| 
                   future::err(SimpleError::new("parse error"))
                );

            SpotifyFuture {
                uri: uri,
                authorization: authorization,
                the_future: Box::new(future),
            }
        }
    }

    impl Future for SpotifyFuture
    where
    {
        type Item = String;
        type Error = SimpleError;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            return self.the_future.poll();
        }
    }
}
