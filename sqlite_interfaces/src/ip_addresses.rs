use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use std::cmp;
use type_flyweight::ip_addresses::IpAddressRateLimit;

fn get_ip_address_rate_limit_from_row(row: &Row) -> Result<IpAddressRateLimit, RusqliteError> {
    Ok(IpAddressRateLimit {
        ip_address: row.get(0)?,
        window_count: row.get(1)?,
        prev_window_count: row.get(2)?,
        updated_at: row.get(3)?,
        deleted_at: row.get(4)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS ip_address_rate_limits (
            ip_address TEXT PRIMARY KEY,
            window_count INTEGER NOT NULL,
            prev_window_count INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER
        )",
        (),
    );

    if let Err(_e) = results {
        return Err("ip_address_rate_limits: failed to create table".to_string());
    }

    Ok(())
}

pub struct RateLimitIpAddressParams<'a> {
    pub ip_address: &'a str,
    pub current_timestamp: u64,
    pub window_limit: u64,
    pub window_length_ms: u64,
}

// return Option<(entry, bool)>
pub fn rate_limit_ip_address(
    conn: &mut Connection,
    params: &RateLimitIpAddressParams,
) -> Result<(bool, IpAddressRateLimit), String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO ip_address_rate_limits
            (ip_address, window_count, prev_window_count, updated_at)
        VALUES
            (?1, 1, 0, ?3)
        ON CONFLICT(ip_address) DO UPDATE
            SET
                window_count =
                    CASE
                        WHEN ?2 < (?3 - updated_at) THEN 1
                        ELSE window_count + 1
                    END,
                prev_window_count =
                    CASE
                        WHEN (2 * ?2) < (?3 - updated_at) THEN 0
                        WHEN ?2 < (?3 - updated_at) THEN window_count 
                        ELSE prev_window_count
                    END,
                updated_at =
                    CASE
                        WHEN ?2 < (?3 - updated_at) THEN ?3
                        ELSE updated_at
                    END
        RETURNING
            *
        ",
    ) {
        Ok(stmt) => stmt,
        Err(_e) => {
            return Err(
                "ip_address_rate_limits: failed to update and return table entry".to_string(),
            )
        }
    };

    let mut ip_address_rate_limit_iter = match stmt.query_map(
        (
            params.ip_address,
            params.window_length_ms,
            params.current_timestamp,
        ),
        get_ip_address_rate_limit_from_row,
    ) {
        Ok(sessions) => sessions,
        _ => {
            return Err(
                "ip_address_rate_limits: failed to create iterable rows from query".to_string(),
            )
        }
    };

    if let Some(ip_address_rate_limit_maybe) = ip_address_rate_limit_iter.next() {
        if let Ok(entry) = ip_address_rate_limit_maybe {
            return Ok((should_rate_limit(&params, &entry), entry));
        }
    }

    Err("ip_address_rate_limits: failed to create or return existing row".to_string())
}

// Use a sliding window counter
fn should_rate_limit(params: &RateLimitIpAddressParams, entry: &IpAddressRateLimit) -> bool {
    let window_delta = params.current_timestamp - entry.updated_at;
    let normalized_delta = cmp::min(1, window_delta / params.window_length_ms);
    let weighted_count = (1 - normalized_delta) * entry.prev_window_count + entry.window_count;

    weighted_count < entry.window_count
}

// delete stale entries
//
// connection.exec returns number of rows from statement
// so delete offset limit
// while returned rows is not less than offset
// (don't use 0 because users can join anytime, don't want a forever process)
pub fn dangerously_delete_stale_entries(
    conn: &mut Connection,
    timestamp: u64,
) -> Result<usize, String> {
    // naive attempt, maybe best attempt?
    match conn.execute(
        "
        DELETE FROM
            ip_address_rate_limits
        WHERE
            updated_at < ?1
        ",
        [timestamp],
    ) {
        Ok(row_count) => Ok(row_count),
        Err(_e) => Err("ip_addres_rate_limits: failed to delete stale entries".to_string()),
    }
}
