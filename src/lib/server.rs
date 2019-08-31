pub mod server {
    use hyper::{Body, Method, Request, Response, Server, StatusCode};
    use hyper::header::{LOCATION};
    use hyper::http::{HeaderValue};

    pub fn redirect<'a>(response: &'a mut Response<Body>, location: &String) -> &'a Response<Body> {
        *response.status_mut() = StatusCode::FOUND;
        println!("{:?}", location.as_str());
        response.headers_mut().append(LOCATION, HeaderValue::from_str(&location.as_str()).unwrap());
        println!("{:?}", response);
        response
    }
}