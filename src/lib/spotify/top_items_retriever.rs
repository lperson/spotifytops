pub struct TopItemRetriever<T> {
    uri: String,
    authorization: String,
}

impl<T> TopItemRetriever<T> {
    fn new(auth_code: &String, what_to_get: &str, timeframe: &str) -> Self {
        let uri = format!(
            "https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}",
            what_to_get, timeframe
        );

        let authorization = format!("Bearer {}", auth_code);

        TopItemRetriever { uri, authorization }
    }
}
