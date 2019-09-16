use super::RetrievableRequest;

pub fn make_request(what_to_get: &str, timeframe: &str) -> RetrievableRequest {
    let uri = format!(
        "https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}",
        what_to_get, timeframe
    );

    RetrievableRequest::new(uri)
}
