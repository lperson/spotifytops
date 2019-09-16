use serde::{Deserialize, Serialize};
use std::vec::Vec;

use super::error_object;
use super::MinimalTrack;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TopTracksResponse {
    error: Option<error_object::ErrorObject>,
    items: Option<Vec<MinimalTrack>>,
}
