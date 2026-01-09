use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use type_flyweight::contacts::Contact;

fn get_entry_from_row(row: &Row) -> Result<Contact, RusqliteError> {
    Ok(Contact {
        id: row.get(0)?,
        people_id: row.get(1)?,
        contact_kind_id: row.get(2)?,
        content: row.get(3)?,
        deleted_at: row.get(4)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            id INTEGER PRIMARY KEY,
            people_id INTEGER NOT NULL,
            contact_kind_id INTEGER NOT NULL,
            content TEXT NOT NULL UNIQUE,
            deleted_at INTEGER,
            UNIQUE (contact_kind_id, content)
        )",
        (),
    );

    if let Err(e) = results {
        return Err("contacts table error: \n".to_string() + &e.to_string());
    }

    Ok(())
}

pub struct CreateParams<'a> {
    pub id: u64,
    pub people_id: u64,
    pub contact_kind_id: u64,
    pub content: &'a str,
}

pub fn create(conn: &mut Connection, params: &CreateParams) -> Result<Contact, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO contacts
            (id, people_id, contact_kind_id, content)
        VALUES
            (?1, ?2, ?3, ?4)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("failed to create a contact".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (
            params.id,
            params.people_id,
            params.contact_kind_id,
            params.content,
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

    Err("failed to create contact".to_string())
}

pub fn read(conn: &mut Connection, id: u64) -> Result<Option<Contact>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            contacts
        WHERE
            deleted_at IS NULL
            AND
            id = ?1
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("failed to read a contact".to_string()),
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

pub struct ReadByKindIdAndContentParams<'a> {
    pub contact_kind_id: u64,
    pub content: &'a str,
}

pub fn read_by_kind_id_and_content(
    conn: &mut Connection,
    params: &ReadByKindIdAndContentParams,
) -> Result<Option<Contact>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            contacts
        WHERE
            deleted_at IS NULL
            AND
            contact_kind_id = ?1
            AND
            content = ?2
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("failed to read a contact by id".to_string()),
    };

    let mut entry_iter =
        match stmt.query_map((params.contact_kind_id, params.content), get_entry_from_row) {
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
