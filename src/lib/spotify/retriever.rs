use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde_json;

pub struct RetrievableRequest {
    pub uri: String,
}

impl RetrievableRequest {
    pub fn new(uri: String) -> Self {
        Self { uri }
    }
}

pub struct Retriever<T> {
    pub retrievable_request: RetrievableRequest,
    pub authorization: String,
    phantom: PhantomData<T>,
}

pub trait Retrievable<'a> {
    type Item: DeserializeOwned;
    fn deserialize(s: &'a str) -> Result<Self::Item, serde_json::Error>;
}

impl<T> Retriever<T> {
    pub fn new(auth_code: &str, retrievable_request: RetrievableRequest) -> Self {
        let authorization = format!("Bearer {}", auth_code);

        Self {
            retrievable_request, 
            authorization,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Retrievable<'a> for Retriever<T>
where
    T: DeserializeOwned,
{
    type Item = T;

    fn deserialize(s: &'a str) -> Result<Self::Item, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(s)
    }
}
