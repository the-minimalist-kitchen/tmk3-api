use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: u64,
    pub people_id: Option<u64>,
    pub deleted_at: Option<u64>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct PublicSession {
    pub id: u64,
    pub people_id: Option<u64>,
    pub token: u64,
    pub session_id: u64,
    pub window_count: u64,
    pub prev_window_count: u64,
    pub updated_at: u64,
    pub deleted_at: Option<u64>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SessionToken {
    pub id: u64,
    pub people_id: Option<u64>,
    pub token: u64,
}
