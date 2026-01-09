use crate::utils::{deserialize_session, get_connection, get_timestamp, set_connection};
use crate::Auth;
use sqlite_interfaces::public_sessions::{
    rate_limit_session, RateLimitParams as RateLimitPublicSessionsParams,
};
use type_flyweight::sessions::{PublicSession, SessionToken};

impl Auth {
    pub fn create_guest_session(&mut self) -> Result<String, String> {
        self.create_session(None)
    }

    pub fn rate_limit_session(
        &mut self,
        session: &str,
    ) -> Result<Option<(bool, PublicSession)>, String> {
        let SessionToken {
            id,
            people_id,
            token,
        } = match deserialize_session(session) {
            Ok(sesh) => sesh,
            Err(e) => return Err(e),
        };

        let current_timestamp = match get_timestamp(&self.params.snowprints) {
            Ok(timestamp) => timestamp,
            Err(e) => return Err(e),
        };

        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };

        // ratelimit
        let public_session_result = rate_limit_session(
            &mut conn,
            &RateLimitPublicSessionsParams {
                window_limit: self.params.ip_address_rate_limit_window_limit,
                window_length_ms: self.params.ip_address_rate_limit_window_length_ms,
                id,
                people_id,
                token,
                current_timestamp,
            },
        );

        // if stale get session
        // if let Ok(public_session_entry) = public_session_result {
        //     if let Some((should_rate_limit, public_session)) {
        //         if !should_rate_limit {
        //             // entry time is bitshifted
        //             // id = snowprints::decompose()
        //             // CURRENT_TIMESTAMP_000 - ID =
        //             if self.params.public_session_lifetime_ms < current_timestamp - ID {

        //                 // read session
        //                 // create new session
        //                 //
        //             }
        //         }
        //     }
        // }

        // set connnection
        if let Err(e) = set_connection(&self.params.connection_pool, conn) {
            return Err(e);
        };

        public_session_result
    }
}
