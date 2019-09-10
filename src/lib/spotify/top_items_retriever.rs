use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde_json;

pub struct Retriever<T> {
    pub uri: String,
    pub authorization: String,
    phantom: PhantomData<T>,
}

pub trait Retrievable<'a> {
    type Item: DeserializeOwned;
    fn deserialize(s: &'a str) -> Result<Self::Item, serde_json::Error>;
}

impl<T> Retriever<T> {
    pub fn new(auth_code: &str, what_to_get: &str, timeframe: &str) -> Self {
        let uri = format!(
            "https://api.spotify.com/v1/me/top/{}?limit=50&time_range={}",
            what_to_get, timeframe
        );

        let authorization = format!("Bearer {}", auth_code);

        Self {
            uri,
            authorization,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Retrievable<'a> for Retriever<T>
where
    T: DeserializeOwned
{
    type Item = T;

    fn deserialize(s: &'a str) -> Result<Self::Item, serde_json::Error>
    where
        T: DeserializeOwned
    {
        serde_json::from_str(s)
    }
}
