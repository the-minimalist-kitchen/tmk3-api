use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct ContactKind {
    pub id: u64,
    pub kind: String,
    pub deleted_at: Option<u64>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Contact {
    pub id: u64,
    pub people_id: u64,
    pub contact_kind_id: u64,
    pub content: String,
    pub deleted_at: Option<u64>,
}
