use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ErrorObject {
    status: u16,
    message: String,
}