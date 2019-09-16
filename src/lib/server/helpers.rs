use std::collections::hash_map::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::vec::Vec;

use hyper::header;
use hyper::http::HeaderValue;
use hyper::{Body, Request, Response, StatusCode};
use futures::future;

use percent_encoding;

use super::ResponseFuture;

pub fn redirect<'a>(response: &'a mut Response<Body>, location: &str) -> &'a Response<Body> {
    *response.status_mut() = StatusCode::FOUND;
    response
        .headers_mut()
        .append(header::LOCATION, HeaderValue::from_str(&location).unwrap());
    response
}

pub fn respond_with_status(status_code: StatusCode) -> Response<Body> {
    let mut response = Response::<Body>::new(Body::empty());
    *response.status_mut() = status_code;
    response
}

pub fn get_query(request: &Request<Body>) -> Option<HashMap<&str, Option<Rc<String>>>> {
    request
        .uri()
        .query()
        .and_then(|uri_query| -> Option<Vec<&str>> { Some(uri_query.split('&').collect()) })
        .and_then(|query_parts| -> Option<HashMap<&str, Option<Rc<String>>>> {
            let query = HashMap::new();
            Some(query_parts.iter().fold(query, |mut query, query_part| {
                let parts: Vec<&str> = query_part.split('=').collect();
                let value = if parts.len() > 1 {
                    Some(Rc::from(String::from(
                        percent_encoding::percent_decode_str(parts[1])
                            .decode_utf8()
                            .unwrap(),
                    )))
                } else {
                    None
                };
                query.insert(parts[0], value);
                query
            }))
        })
}

// https://github.com/brson/basic-http-server/blob/9577fd13c09838c884589189f909a5cea7bde462/src/main.rs#L248
pub fn file_path_mime(file_path: &Path) -> mime::Mime {
    match file_path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("html") => mime::TEXT_HTML,
        Some("css") => mime::TEXT_CSS,
        Some("js") => mime::TEXT_JAVASCRIPT,
        Some("jpg") => mime::IMAGE_JPEG,
        Some("md") => "text/markdown; charset=UTF-8"
            .parse::<mime::Mime>()
            .unwrap(),
        Some("png") => mime::IMAGE_PNG,
        Some("svg") => mime::IMAGE_SVG,
        Some("wasm") => "application/wasm".parse::<mime::Mime>().unwrap(),
        _ => mime::TEXT_PLAIN,
    }
}
