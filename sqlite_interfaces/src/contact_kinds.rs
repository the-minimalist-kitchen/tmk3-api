use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use type_flyweight::contacts::ContactKind;

fn get_entry_from_row(row: &Row) -> Result<ContactKind, RusqliteError> {
    Ok(ContactKind {
        id: row.get(0)?,
        kind: row.get(1)?,
        deleted_at: row.get(2)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS contact_kinds (
            id INTEGER PRIMARY KEY,
            kind TEXT NOT NULL UNIQUE,
            deleted_at INTEGER
        )",
        (),
    );

    if let Err(e) = results {
        return Err("contact_kinds table error: \n".to_string() + &e.to_string());
    }

    Ok(())
}

pub fn create(conn: &mut Connection, id: u64, kind: &str) -> Result<ContactKind, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO contact_kinds
            (id, kind)
        VALUES
            (?1, ?2)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not prepare statement".to_string()),
    };

    let mut entry_iter = match stmt.query_map((id, kind), get_entry_from_row) {
        Ok(entry) => entry,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(entry);
        }
    }

    Err("failed to create contact kind".to_string())
}

pub fn read(conn: &mut Connection, id: u64) -> Result<Option<ContactKind>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            contact_kinds
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
        Ok(entry) => entry,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(Some(entry));
        }
    }

    Ok(None)
}

pub fn read_by_kind(conn: &mut Connection, kind: &str) -> Result<Option<ContactKind>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            contact_kinds
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
        Ok(entry) => entry,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(Some(entry));
        }
    }

    Ok(None)
}
