use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use type_flyweight::dangerous_actions::DangerousActionKind;

fn get_entry_from_row(row: &Row) -> Result<DangerousActionKind, RusqliteError> {
    Ok(DangerousActionKind {
        id: row.get(0)?,
        kind: row.get(1)?,
        lifetime_ms: row.get(2)?,
        deleted_at: row.get(3)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS dangerous_action_kinds (
            id INTEGER PRIMARY KEY,
            kind TEXT NOT NULL UNIQUE,
            lifetime_ms INTEGER,
            deleted_at INTEGER
        )",
        (),
    );

    if let Err(e) = results {
        return Err("dangerous_action_kinds table error: \n".to_string() + &e.to_string());
    }

    Ok(())
}

pub fn create(
    conn: &mut Connection,
    id: u64,
    kind: &str,
    lifetime_ms: u64,
) -> Result<DangerousActionKind, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO dangerous_action_kinds
            (id, kind, lifetime_ms)
        VALUES
            (?1, ?2, ?3)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map((id, kind, lifetime_ms), get_entry_from_row) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(entry);
        }
    }

    Err("failed to create dangerous action kind".to_string())
}

pub fn read(conn: &mut Connection, id: u64) -> Result<Option<DangerousActionKind>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            dangerous_action_kinds
        WHERE
            deleted_at IS NULL
            AND
            id = ?1
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map([id], get_entry_from_row) {
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

pub fn read_by_kind(
    conn: &mut Connection,
    kind: &str,
) -> Result<Option<DangerousActionKind>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            dangerous_action_kinds
        WHERE
            deleted_at IS NULL
            AND
            kind = ?1
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map([kind], get_entry_from_row) {
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
