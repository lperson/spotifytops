use chrono::{DateTime, offset::Utc};

use serde::{Deserialize, Serialize};
use std::vec::Vec;

use super::{ErrorObject, MinimalTrack};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayHistoryContext {
    r#type: String
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayHistoryItem {
    track: MinimalTrack,
    played_at: DateTime<Utc>,
    context: Option<PlayHistoryContext>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RecentlyPlayedResponse {
    error: Option<ErrorObject>,
    items: Option<Vec<PlayHistoryItem>>,
}
