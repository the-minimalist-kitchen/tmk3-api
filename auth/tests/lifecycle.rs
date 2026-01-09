use auth;
use snowprints;
use sqlite_connection_pool;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[test]
fn api_check() {
    let snowprints = match snowprints::Snowprints::from(snowprints::Params {
        logical_volume_base: 0,
        logical_volume_length: 8192,
        origin_time_ms: 0,
    }) {
        Ok(snowprints) => snowprints,
        Err(e) => return assert!(false, "{:?}", e),
    };

    let conn_pool =
        sqlite_connection_pool::ConnectionPool::from_params(sqlite_connection_pool::Params {
            db_filepath: PathBuf::from("./auth-rs-tests.sqlite"),
            connection_limit: 4,
        });

    let snowprints_arcd = Arc::new(Mutex::new(snowprints));
    let conn_pool_arcd = Arc::new(Mutex::new(conn_pool));

    let auth = auth::Auth::from(auth::Params {
        snowprints: snowprints_arcd.clone(),
        connection_pool: conn_pool_arcd.clone(),
        ip_address_rate_limit_window_limit: 1024,
        ip_address_rate_limit_window_length_ms: 1024,
        public_session_lifetime_ms: 1000 * 60 * 60 * 24,
    });
}
