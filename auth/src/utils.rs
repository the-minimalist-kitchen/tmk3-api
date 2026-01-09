use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use rand::Rng;
use rusqlite::Connection;
use snowprints::Snowprints;
use sqlite_connection_pool::ConnectionPool;
use std::sync::{Arc, Mutex};
use type_flyweight::sessions::SessionToken;

pub struct TwoSnowflakeIdsAndTimestamp {
    pub session_id: u64,
    pub public_session_id: u64,
    pub current_timestamp: u64,
}

pub fn get_timestamp(snowprints_arcd: &Arc<Mutex<Snowprints>>) -> Result<u64, String> {
    let snowprints = match snowprints_arcd.lock() {
        Ok(snow) => snow,
        Err(_e) => return Err("couldnt lock snowprints".to_string()),
    };

    Ok(snowprints.get_timestamp())
}

pub fn get_connection(
    connection_pool_arcd: &Arc<Mutex<ConnectionPool>>,
) -> Result<Connection, String> {
    let mut connection_pool = match connection_pool_arcd.lock() {
        Ok(pool) => pool,
        Err(_e) => return Err("couldnt lock snowprints".to_string()),
    };

    connection_pool.get_connection()
}

pub fn set_connection(
    connection_pool_arcd: &Arc<Mutex<ConnectionPool>>,
    conn: Connection,
) -> Result<(), String> {
    let mut connection_pool = match connection_pool_arcd.lock() {
        Ok(pool) => pool,
        Err(_e) => return Err("couldnt lock snowprints".to_string()),
    };

    connection_pool.set_connection(conn)
}

pub fn get_snowflake_id(snowprints_arcd: &Arc<Mutex<Snowprints>>) -> Result<u64, String> {
    let mut snowprints = match snowprints_arcd.lock() {
        Ok(snow) => snow,
        Err(_e) => return Err("couldnt lock snowprints".to_string()),
    };

    match snowprints.create_id() {
        Ok(id) => Ok(id),
        Err(_e) => return Err("could not create id".to_string()),
    }
}

pub fn get_two_snowflake_ids_and_timestamp(
    snowprints_arcd: &Arc<Mutex<Snowprints>>,
) -> Result<TwoSnowflakeIdsAndTimestamp, String> {
    let mut snowprints = match snowprints_arcd.lock() {
        Ok(snow) => snow,
        Err(_e) => return Err("couldnt lock snowprints".to_string()),
    };

    let session_id = match snowprints.create_id() {
        Ok(id) => id,
        Err(_e) => return Err("could not create id".to_string()),
    };

    let public_session_id = match snowprints.create_id() {
        Ok(id) => id,
        Err(_e) => return Err("could not create id".to_string()),
    };

    let current_timestamp = snowprints.get_timestamp();

    return Ok(TwoSnowflakeIdsAndTimestamp {
        session_id,
        public_session_id,
        current_timestamp,
    });
}

// function create a session with random u64
pub fn create_token_u64() -> u64 {
    let mut rng = rand::rng();
    rng.random()
}

pub fn serialize_session_token(session_token: &SessionToken) -> String {
    let mut session = "".to_string();

    let SessionToken {
        id,
        people_id,
        token,
    } = session_token;

    session.push_str(&URL_SAFE.encode(id.to_ne_bytes()));
    session.push(':');

    if let Some(peop_id) = people_id {
        session.push_str(&URL_SAFE.encode(peop_id.to_ne_bytes()));
    }

    session.push(':');
    session.push_str(&URL_SAFE.encode(token.to_ne_bytes()));

    session
}

// function to get (id, people_id, token)
pub fn deserialize_session(session: &str) -> Result<SessionToken, String> {
    let parts: Vec<&str> = session.split(":").collect();
    if parts.len() != 3 {
        return Err("invalid token".to_string());
    }

    let id = decode_u64_from_base64(parts[0]);
    let mut people_id = None;
    if "" != parts[1] {
        people_id = decode_u64_from_base64(parts[1]);
    }
    let token = decode_u64_from_base64(parts[2]);

    if let (Some(id), Some(token)) = (id, token) {
        return Ok(SessionToken {
            id,
            people_id,
            token,
        });
    }

    Err("failed to digest session".to_string())
}

// Might be better to fail and err.
// But all three values will be compared later.
fn decode_u64_from_base64(base64_str: &str) -> Option<u64> {
    if let Ok(invitation_vec_bytes) = URL_SAFE.decode(base64_str.as_bytes()) {
        if let Ok(invitation_arr) = get_arry_u8(invitation_vec_bytes) {
            return Some(u64::from_ne_bytes(invitation_arr));
        }
    }

    None
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(ph) => Ok(ph.to_string()),
        Err(e) => return Err(e.to_string()),
    }
}

pub fn validate_password(password_hash_results: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(&password_hash_results) {
        Ok(ph) => ph,
        _ => return false,
    };

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => true,
        _ => false,
    }
}

fn get_arry_u8(data_vec: Vec<u8>) -> Result<[u8; 8], String> {
    if 8 != data_vec.len() {
        return Err("required length not found".to_string());
    }

    let mut data: [u8; 8] = [0; 8];
    let mut index = 0;
    for pip in data {
        data[index] = pip;
        index += 1;
    }

    Ok(data)
}

// fn get_invitation_and_session_from_base64(invitation_base64: &str) -> Result<(u64, u64), String> {
//     let mut splitted = invitation_base64.split(":");

//     let mut invitation_u64: Option<u64> = None;
//     if let Some(invitation_base64) = splitted.next() {
//         if let Ok(invitation_vec_bytes) = URL_SAFE.decode(invitation_base64.as_bytes()) {
//             if let Ok(invitation_arr) = get_arry_u8(invitation_vec_bytes) {
//                 invitation_u64 = Some(u64::from_ne_bytes(invitation_arr));
//             }
//         }
//     }

//     let mut session_u64: Option<u64> = None;
//     if let Some(session_base64) = splitted.next() {
//         if let Ok(session_vec_bytes) = URL_SAFE.decode(session_base64.as_bytes()) {
//             if let Ok(session_arr) = get_arry_u8(session_vec_bytes) {
//                 session_u64 = Some(u64::from_ne_bytes(session_arr));
//             }
//         };
//     }

//     if let (Some(invitation), Some(session)) = (invitation_u64, session_u64) {
//         return Ok((invitation, session));
//     }

//     Err("didnt' make it!".to_string())
// }
