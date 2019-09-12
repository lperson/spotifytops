use serde::Serialize;

use super::super::super::CONFIG;
use super::{get_callback};

#[derive(Serialize, Debug)]
pub struct TokenRequest<'a> {
    grant_type: String,
    client_id: &'a String,
    client_secret: &'a String,
}

impl TokenRequest<'_> {
    pub fn get_serialized_request(code: &str) -> String {
        let mut token_request = serde_urlencoded::to_string(TokenRequest {
            grant_type: "authorization_code".to_string(),
            client_id: &CONFIG.client_id,
            client_secret: &CONFIG.client_secret,
        })
        .unwrap();

        token_request.push_str(&format!("&code={}&redirect_uri={}", code, get_callback()));

        token_request
    }
}
