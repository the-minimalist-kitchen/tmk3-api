use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Totp {
    pub id: u64,
    pub people_id: u64,
    pub secret_key: String,
    pub algorithm: Option<u64>,
    pub period: Option<u64>,
    pub digits: Option<u64>,
    pub deleted_at: Option<u64>,
}
