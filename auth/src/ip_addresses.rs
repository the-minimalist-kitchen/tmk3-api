use crate::utils::{get_connection, get_timestamp, set_connection};
use crate::Auth;
use sqlite_interfaces::ip_addresses::{rate_limit_ip_address, RateLimitIpAddressParams};
use type_flyweight::ip_addresses::IpAddressRateLimit;

// whatif just pass connection?
impl Auth {
    pub fn rate_limit_ip_address(
        &mut self,
        ip_address: &str,
    ) -> Result<(bool, IpAddressRateLimit), String> {
        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };

        // get current timestamp
        let current_timestamp = match get_timestamp(&self.params.snowprints) {
            Ok(timestamp) => timestamp,
            Err(e) => return Err(e),
        };

        // ratelimit
        let entry_result = rate_limit_ip_address(
            &mut conn,
            &RateLimitIpAddressParams {
                window_limit: self.params.ip_address_rate_limit_window_limit,
                window_length_ms: self.params.ip_address_rate_limit_window_length_ms,
                current_timestamp,
                ip_address,
            },
        );

        // set connnection
        if let Err(e) = set_connection(&self.params.connection_pool, conn) {
            return Err(e);
        };

        // return results
        entry_result
    }
}
