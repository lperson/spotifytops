use serde::Deserialize;
use std::vec::Vec;

use super::error_object;


#[derive(Deserialize, Debug)]
pub struct TopArtist {
    name: String
}

#[derive(Deserialize, Debug)]
pub struct TopArtistResponse {
    error: Option<error_object::ErrorObject>,
    items: Option<Vec<TopArtist>>,
}

