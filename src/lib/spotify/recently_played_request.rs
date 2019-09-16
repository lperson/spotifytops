use super::RetrievableRequest;

pub fn make_request() -> RetrievableRequest {
    let uri = String::from("https://api.spotify.com/v1/me/player/recently-played?limit=50");
    RetrievableRequest::new(uri)
}
