pub mod spotify_future {

    use futures::{future, future::finished, prelude::*, Future, Poll };
    use hyper::Client;
    use hyper::{Body, Request, Response};
    use hyper_tls::HttpsConnector;
    use simple_error::SimpleError;

    type BoxFut = Box<dyn Future<Item = String, Error = SimpleError> + Send>;

    pub struct SpotifyFutureConfig {
        uri: String,
        authorization: String,
    }
    
    pub struct SpotifyFuture {
        config: SpotifyFutureConfig,
        the_future: BoxFut,
        count: u8,
    }

    impl SpotifyFuture {
        fn make_future(config: &SpotifyFutureConfig) -> BoxFut {
            let https = HttpsConnector::new(4).unwrap();
            let client = Client::builder().build::<_, hyper::Body>(https);
            let request = Request::builder()
                .method("GET")
                .uri(config.uri.clone())
                .header("Authorization", config.authorization.clone())
                .body(Body::empty())
                .unwrap();

            let future = client
                .request(request)
                .map_err(|x| SimpleError::new("x"))
                .and_then(move |result| {
                    println!("{}", result.status());
                    if [404u16, 400u16].into_iter().any(|x| result.status().as_u16() == *x ) {
                        let this_return = future::err(SimpleError::new("400"));
                        return this_return;
                            //.map(|x| x)
                            //.map_err(|x| x);
                    }

                    return future::ok(result);
                })
                .and_then(|result| {
                    let transformed_result = result
                        .into_body()
                        .concat2()
                        .map(|x| String::from_utf8(x.to_vec()).unwrap())
                        .map_err(|x| SimpleError::new("x"));

                    return transformed_result;
                }
                );
            return Box::new(future)
        } 

        pub fn new(auth_code: &String, what_to_get: &str, timeframe: &str) -> Self {

            let uri = format!(
                    "https://api.spotify.com/v1/me/topx/{}?limit=50&time_range={}",
                    what_to_get, timeframe
                );

            let authorization = format!("Bearer {}", auth_code);

            let config = SpotifyFutureConfig {
                uri, authorization
            };

            let the_future = Self::make_future(&config);

            SpotifyFuture {
                config,
                the_future,
                count: 0, 
            }
        }
    }

    impl Future for SpotifyFuture
    where
    {
        type Item = String;
        type Error = SimpleError;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            match self.the_future.poll() {
                Ok(Async::Ready(t)) => {Ok(Async::Ready(t))}
                Ok(Async::NotReady) => {Ok(Async::NotReady)}
                Err(e) => { 
                    println!("IN POLL ==> {:?}", e);
                    if e.as_str() == "400" && self.count == 1 {
                        println!("400 in poll");
                        return Err(e);
                    }

                    self.the_future = Self::make_future(&self.config);
                    self.count = 1;
                    return self.the_future.poll();
                }

            }
        }
    }
}
