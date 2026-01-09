mod contacts;
mod ip_addresses;
mod people;
mod sessions;
mod utils;

use crate::utils::{
    create_token_u64, get_connection, get_two_snowflake_ids_and_timestamp, serialize_session_token,
    set_connection, TwoSnowflakeIdsAndTimestamp,
};
use snowprints::Snowprints;
use sqlite_connection_pool::ConnectionPool;
use sqlite_interfaces::public_sessions::{
    create as create_public_session, CreateParams as CreatePublicSessionParams,
};
use sqlite_interfaces::sessions::{create as create_session, CreateParams as CreateSessionParams};
use std::sync::{Arc, Mutex};

use type_flyweight::sessions::SessionToken;

pub struct Params {
    pub snowprints: Arc<Mutex<Snowprints>>,
    pub connection_pool: Arc<Mutex<ConnectionPool>>,

    pub public_session_lifetime_ms: u64,
    pub ip_address_rate_limit_window_length_ms: u64,
    pub ip_address_rate_limit_window_limit: u64,
}

pub struct Auth {
    params: Params,
}

// Private functions need to be on the source impl apparently

impl Auth {
    pub fn from(params: Params) -> Auth {
        Auth { params }
    }

    fn create_session(&mut self, people_id: Option<u64>) -> Result<String, String> {
        // Create id and get timestamp, one function one lock
        let TwoSnowflakeIdsAndTimestamp {
            session_id,
            public_session_id,
            current_timestamp,
        } = match get_two_snowflake_ids_and_timestamp(&self.params.snowprints) {
            Ok(tple) => tple,
            Err(e) => return Err(e),
        };

        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };
        
        // create session
        if let Err(e) = create_session(
            &mut conn,
            &CreateSessionParams {
                id: session_id,
                people_id,
            },
        ) {
            return Err(e);
        };
        
        let token = create_token_u64();
        let public_session_result = create_public_session(
            &mut conn,
            &CreatePublicSessionParams {
                id: public_session_id,
                people_id,
                session_id,
                token,
                current_timestamp,
            },
        );

        // set connnection
        if let Err(e) = set_connection(&self.params.connection_pool, conn) {
            return Err(e);
        };

        // use entry
        if let Ok(entry) = public_session_result {
            return Ok(serialize_session_token(&SessionToken {
                id: entry.id,
                people_id: entry.people_id,
                token: entry.token,
            }));
        }

        Err("one moment please".to_string())
    }
}
