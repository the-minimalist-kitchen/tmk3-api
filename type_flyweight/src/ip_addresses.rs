use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct IpAddressRateLimit {
    pub ip_address: String,
    pub window_count: u64,
    pub prev_window_count: u64,
    pub session_count: u64,
    pub prev_session_count: u64,
    pub updated_at: u64,
    pub deleted_at: Option<u64>,
}
