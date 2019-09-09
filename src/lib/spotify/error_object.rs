use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ErrorObject {
    status: u16,
    message: String,
}