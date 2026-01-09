// 0 Administrator
// 1 Maintainer

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct RoleKind {
    pub id: u64,
    pub kind: String,
    pub deleted_at: Option<u64>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Roles {
    pub id: u64,
    pub role_kind_id: u64,
    pub people_id: u64,
    pub deleted_at: Option<u64>,
}
