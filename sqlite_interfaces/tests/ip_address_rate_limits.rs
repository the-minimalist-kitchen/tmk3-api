use rusqlite::{Connection, Result};
use sqlite_interfaces::ip_addresses::{
    create_table, dangerously_delete_stale_entries, rate_limit_ip_address, RateLimitIpAddressParams,
};

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(e) = create_table(&mut conn) {
        assert!(false, "{}", e);
    }

    let params = RateLimitIpAddressParams {
        ip_address: "127.0.0.1",
        current_timestamp: 20,
        window_limit: 10,
        window_length_ms: 5,
    };

    // create
    let (should_rate_limit, entry) = match rate_limit_ip_address(&mut conn, &params) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(
        !should_rate_limit,
        "should not have rate limited ip address"
    );

    // update
    let params = RateLimitIpAddressParams {
        ip_address: "127.0.0.1",
        current_timestamp: 22,
        window_limit: 10,
        window_length_ms: 5,
    };

    let (should_rate_limit, second_entry) = match rate_limit_ip_address(&mut conn, &params) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(
        !should_rate_limit,
        "should not have rate limited ip address on second try"
    );

    assert!(entry != second_entry);

    // delete all
    //
    let entry_count = match dangerously_delete_stale_entries(&mut conn, 21) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(entry_count != 0, "did not delete session entry");

    // assert limit
    Ok(())
}
