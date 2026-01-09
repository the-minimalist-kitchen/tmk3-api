use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use std::cmp;
use type_flyweight::sessions::{PublicSession, SessionToken};

fn get_entry_from_row(row: &Row) -> Result<PublicSession, RusqliteError> {
    Ok(PublicSession {
        id: row.get(0)?,
        people_id: row.get(1)?,
        token: row.get(2)?,
        session_id: row.get(3)?,
        window_count: row.get(4)?,
        prev_window_count: row.get(5)?,
        updated_at: row.get(6)?,
        deleted_at: row.get(7)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS public_sessions (
            id INTEGER PRIMARY KEY,
            people_id INTEGER,
            token INTEGER NOT NULL,
            session_id INTEGER NOT NULL,
            window_count INTEGER NOT NULL,
            prev_window_count INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER
        )",
        (),
    );

    if let Err(e) = results {
        return Err("sessions table error: \n".to_string() + &e.to_string());
    }

    Ok(())
}

pub struct CreateParams {
    pub id: u64,
    pub people_id: Option<u64>,
    pub token: u64,
    pub session_id: u64,
    pub current_timestamp: u64,
}

pub fn create(conn: &mut Connection, params: &CreateParams) -> Result<PublicSession, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO public_sessions
            (id, people_id, token, session_id, window_count, prev_window_count, updated_at)
        VALUES
            (?1, ?2, ?3, ?4, 1, 0, ?5)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (
            params.id,
            params.people_id,
            params.token,
            params.session_id,
            params.current_timestamp,
        ),
        get_entry_from_row,
    ) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(entry);
        }
    }

    Err("failed to create public session".to_string())
}

pub fn read(conn: &mut Connection, params: &SessionToken) -> Result<Option<PublicSession>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            public_sessions
        WHERE
            deleted_at IS NULL
            AND
            id = ?1
            AND
            people_id = ?2
            AND
            token = ?3
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (params.id, params.people_id, params.token),
        get_entry_from_row,
    ) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(Some(entry));
        }
    }

    Ok(None)
}

pub struct ReadAllByIdParams {
    pub id: u64,
    pub limit: u64,
    pub offset: u64,
}

pub fn read_all_by_session_id(
    conn: &mut Connection,
    params: &ReadAllByIdParams,
) -> Result<Vec<PublicSession>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            public_sessions
        WHERE
            deleted_at IS NULL
            AND
            session_id = ?1
        ORDER BY
            id DESC
        LIMIT
            ?2,?3
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let entry_iter =
        match stmt.query_map((params.id, params.offset, params.limit), get_entry_from_row) {
            Ok(entry_iter) => entry_iter,
            Err(e) => return Err(e.to_string()),
        };

    let mut sessions: Vec<PublicSession> = Vec::new();
    for entry_maybe in entry_iter {
        if let Ok(entry) = entry_maybe {
            sessions.push(entry);
        }
    }

    Ok(sessions)
}

pub struct ReadAllByPeopleIdParams {
    pub people_id: Option<u64>,
    pub limit: u64,
    pub offset: u64,
}

pub fn read_all_by_people_id(
    conn: &mut Connection,
    params: &ReadAllByPeopleIdParams,
) -> Result<Vec<PublicSession>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            public_sessions
        WHERE
            deleted_at IS NULL
            AND
            people_id = ?1
        ORDER BY
            id DESC
        LIMIT
            ?2,?3
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let entry_iter = match stmt.query_map(
        (params.people_id, params.offset, params.limit),
        get_entry_from_row,
    ) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut sessions: Vec<PublicSession> = Vec::new();
    for entry_maybe in entry_iter {
        if let Ok(entry) = entry_maybe {
            sessions.push(entry);
        }
    }

    Ok(sessions)
}

pub struct RateLimitParams {
    pub id: u64,
    pub people_id: Option<u64>,
    pub token: u64,
    pub current_timestamp: u64,
    pub window_limit: u64,
    pub window_length_ms: u64,
}

pub fn rate_limit_session(
    conn: &mut Connection,
    params: &RateLimitParams,
) -> Result<Option<(bool, PublicSession)>, String> {
    // update or ignore

    // provide id, window limit, window length, and current_timestamp
    let mut stmt = match conn.prepare(
        "
        UPDATE OR IGNORE public_sessions
            SET
                window_count =
                    CASE
                        WHEN ?1 < (?2 - updated_at) THEN 1
                        ELSE window_count + 1
                    END,
                prev_window_count =
                    CASE
                        WHEN (2 * ?1) < (?2 - updated_at) THEN 0
                        WHEN ?1 < (?2 - updated_at) THEN window_count 
                        ELSE prev_window_count
                    END,
                updated_at =
                    CASE
                        WHEN ?1 < (?2 - updated_at) THEN ?2
                        ELSE updated_at
                    END
            WHERE
                deleted_at IS NULL
                AND
                id = ?3
                AND
                people_id = ?4
                AND
                token = ?5
        RETURNING
            *
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (
            params.window_length_ms,
            params.current_timestamp,
            params.id,
            params.people_id,
            params.token,
        ),
        get_entry_from_row,
    ) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(Some((should_rate_limit(&params, &entry), entry)));
        }
    }

    Ok(None)
}

fn should_rate_limit(params: &RateLimitParams, entry: &PublicSession) -> bool {
    let window_delta = params.current_timestamp - entry.updated_at;
    let normalized_delta = cmp::min(1, window_delta / params.window_length_ms);
    let weighted_count = (1 - normalized_delta) * entry.prev_window_count + entry.window_count;

    weighted_count < entry.window_count
}

pub struct DeleteAllParams {
    pub session_id: u64,
    pub current_timestamp: u64,
}

pub fn delete_all(
    conn: &mut Connection,
    params: &DeleteAllParams,
) -> Result<Vec<PublicSession>, String> {
    // provide id, window limit, window length, and current_timestamp
    let mut stmt = match conn.prepare(
        "
        UPDATE OR IGNORE public_sessions
            SET deleted_at = ?1
            WHERE
                deleted_at IS NULL
                AND
                session_id = ?2
        RETURNING
            *
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (params.current_timestamp, params.session_id),
        get_entry_from_row,
    ) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut entries: Vec<PublicSession> = Vec::new();

    while let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            entries.push(entry);
        }
    }

    Ok(entries)
}
