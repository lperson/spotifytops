extern crate percent_encoding;
use percent_encoding::{percent_encode, CONTROLS};

pub mod spotify_scopes;
use super::super::CONFIG;

pub mod token_request;
pub mod token_response;

use std::io::Write;
use std::vec::Vec;

pub fn p_encode(s: &String) -> String {
    percent_encode(s.as_bytes(), CONTROLS).to_string()
}

pub fn get_callback() -> String {
    let mut callback: Vec<u8> = Vec::new();
    write!(
        callback,
        "{}/SpotifyLoginCallback/",
        p_encode(&CONFIG.redirect_host_and_port)
    )
    .unwrap();
    String::from_utf8(callback).unwrap()
}

pub fn get_redirect(state: &String) -> String {
    let mut redirect: Vec<u8> = Vec::new();
    write!(
        redirect,
        "https://accounts.spotify.com/authorize/?client_id={}\
         &response_type=code&redirect_uri={}&scope={}&state={}&show_dialog=False",
        p_encode(&CONFIG.client_id),
        p_encode(&get_callback()),
        p_encode(&spotify_scopes::spotify_scopes::get_scopes()),
        p_encode(&state)
    )
    .unwrap();
    String::from_utf8(redirect).unwrap()
}
