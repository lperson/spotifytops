use serde::{Deserialize, Serialize};

use super::MinimalArtist;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MinimalTrack {
    name: String,
    artists: Vec<MinimalArtist>,
}