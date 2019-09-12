use std::collections::hash_map::HashMap;
use std::rc::Rc;
use std::vec::Vec;

use hyper::header::LOCATION;
use hyper::http::HeaderValue;
use hyper::{Body, Request, Response, StatusCode};

use percent_encoding;

pub fn redirect<'a>(response: &'a mut Response<Body>, location: &str) -> &'a Response<Body> {
    *response.status_mut() = StatusCode::FOUND;
    response
        .headers_mut()
        .append(LOCATION, HeaderValue::from_str(&location).unwrap());
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
