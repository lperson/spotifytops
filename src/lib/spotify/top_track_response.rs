use serde::{Deserialize, Serialize};
use std::vec::Vec;

use super::error_object;
use super::top_artist_response::TopArtist;


#[derive(Serialize, Deserialize, Debug)]
pub struct TopTrack {
    name: String,
    artists: Vec<TopArtist>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopTrackResponse {
    error: Option<error_object::ErrorObject>,
    items: Option<Vec<TopTrack>>,
}

