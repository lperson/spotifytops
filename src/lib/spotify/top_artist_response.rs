use serde::{Deserialize, Serialize};
use std::vec::Vec;

use super::error_object;
use super::MinimalArtist;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TopArtistResponse {
    error: Option<error_object::ErrorObject>,
    items: Option<Vec<MinimalArtist>>,
}