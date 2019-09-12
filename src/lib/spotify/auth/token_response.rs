use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AuthenticationError {
    pub error: String,
    pub error_description: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct TokenResponse {
    pub error: Option<AuthenticationError>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<u32>,
    pub scope: Option<String>,
}

impl TokenResponse {
    pub fn new_error(error: String, error_description: String) -> TokenResponse {
        TokenResponse {
            error: Some(AuthenticationError {
                error,
                error_description,
            }),
            access_token: None,
            refresh_token: None,
            token_type: None,
            expires_in: None,
            scope: None,
        }
    }
}
