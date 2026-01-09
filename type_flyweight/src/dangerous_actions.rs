// 0 create account
// 1 update password
// 2 create contact
// 3 recover account
// 4 delete account

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct DangerousActionKind {
    pub id: u64,
    pub kind: String,
    pub lifetime_ms: u64,
    pub deleted_at: Option<u64>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct DangerousAction {
    pub id: u64,
    pub people_id: Option<u64>,
    pub token: u64,
    pub dangerous_action_kind_id: u64,
    pub contact_kind_id: u64,
    pub contact_content: String,
    pub deleted_at: Option<u64>,
}
