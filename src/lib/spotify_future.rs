pub mod spotify_future {

    use futures::{future, future::finished, prelude::*, Future, Poll};
    use hyper::Client;
    use hyper::{Body, Request, Response};
    use hyper_tls::HttpsConnector;

    type BoxFut = Box<dyn Future<Item = hyper::Chunk, Error = hyper::Error> + Send>;
    
    pub struct SpotifyFuture {
        uri: String,
        authorization: String,
        the_future: BoxFut,
    }

    impl SpotifyFuture {
        pub fn new(auth_code: &String, what_to_get: &str, timeframe: &str) -> Self {
            let https = HttpsConnector::new(4).unwrap();
            let client = Client::builder().build::<_, hyper::Body>(https);

            let request = Request::builder()
                .method("GET")
                .uri(format!(
                    "https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}",
                    "tracks", "short_term"
                ))
                .header("Authorization", format!("Bearer {}", auth_code))
                .body(Body::empty())
                .unwrap();

            let future = client
                .request(request)
                .and_then(move |result| result.into_body().concat2());

            SpotifyFuture {
                uri: format!(
                    "https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}",
                    "tracks", "short_term"
                ),
                authorization: format!("Bearer {}", auth_code),
                the_future: Box::new(future),
            }
        }
    }

    impl Future for SpotifyFuture
    where
    {
        type Item = hyper::Chunk;
        type Error = hyper::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            return self.the_future.poll();
        }
    }
}
