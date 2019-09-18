use hyper::{header::HeaderName, header, http::HeaderValue, Body, Response as HyperResponse, StatusCode};

pub struct Response<B> {
    hyper_response: HyperResponse<B>,
}

impl Response<Body> {
    pub fn new(hyper_response: HyperResponse<Body>) -> Self {
        Self { hyper_response }
    }

    pub fn from_string(body: String) -> Self {
        let hyper_response = HyperResponse::new(Body::from(body));
        Self {
            hyper_response
        }
    }

    pub fn with_status(status_code: StatusCode) -> Self {
        let mut hyper_response = HyperResponse::new(Body::empty());
        *hyper_response.status_mut() = status_code;
        Self { hyper_response }
    }

    pub fn with_redirect(location: &str) -> Self {
        let mut response = Self::with_status(StatusCode::FOUND);
        response
            .hyper_response
            .headers_mut()
            .append(header::LOCATION, HeaderValue::from_str(&location).unwrap());
        response
    }

    pub fn set_cookie(&mut self, cookie_name: &str, cookie_value: &str) -> &Self {
        self.hyper_response.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_str(format!("{}:{}", cookie_name, cookie_value).as_str()).unwrap(),
        );
        self
    }

    pub fn set_header(&mut self, header_name: HeaderName, header_value: &str) -> &Self {
        self.hyper_response.headers_mut().insert(
            header_name,
            HeaderValue::from_str(header_value).unwrap(),
        );
        self
    }
}

impl Into<HyperResponse<Body>> for Response<Body> {
    fn into(self) -> HyperResponse<Body> {
        self.hyper_response
    }
}
