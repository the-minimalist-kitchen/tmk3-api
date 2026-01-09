use serde::{Deserialize, Serialize};

// window count

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Person {
    pub id: u64,
    pub password_hash_results: String,
    pub window_count: u64,
    pub prev_window_count: u64,
    pub updated_at: u64,
    pub deleted_at: Option<u64>,
}
