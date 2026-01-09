use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use type_flyweight::sessions::Session;

fn get_entry_from_row(row: &Row) -> Result<Session, RusqliteError> {
    Ok(Session {
        id: row.get(0)?,
        people_id: row.get(1)?,
        deleted_at: row.get(2)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY,
            people_id INTEGER,
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
}

pub fn create(conn: &mut Connection, params: &CreateParams) -> Result<Session, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO sessions
            (id, people_id)
        VALUES
            (?1, ?2)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map((params.id, params.people_id), get_entry_from_row) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(entry);
        }
    }

    Err("failed to create session".to_string())
}

pub fn read(conn: &mut Connection, session_id: u64) -> Result<Option<Session>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            sessions
        WHERE
            id = ?1
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map([session_id], get_entry_from_row) {
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

pub struct ReadAllByPeopleIdParams {
    pub people_id: Option<u64>,
    pub offset: usize,
    pub limit: usize,
}

pub fn read_all_by_people_id(
    conn: &mut Connection,
    params: &ReadAllByPeopleIdParams,
) -> Result<Vec<Session>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            sessions
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

    let session_iter = match stmt.query_map(
        (params.people_id, params.offset, params.limit),
        get_entry_from_row,
    ) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut sessions: Vec<Session> = Vec::new();
    for entry_maybe in session_iter {
        if let Ok(entry) = entry_maybe {
            sessions.push(entry);
        }
    }

    Ok(sessions)
}

pub struct DeleteParams {
    pub id: u64,
    pub current_timestamp: u64,
}

pub fn delete(conn: &mut Connection, params: &DeleteParams) -> Result<Option<Session>, String> {
    // provide id, window limit, window length, and current_timestamp
    let mut stmt = match conn.prepare(
        "
        UPDATE OR IGNORE sessions
            SET deleted_at = ?1
            WHERE
                deleted_at IS NULL
                AND
                id = ?2
        RETURNING
            *
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter =
        match stmt.query_map((params.current_timestamp, params.id), get_entry_from_row) {
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
