use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use type_flyweight::dangerous_actions::DangerousAction;

fn get_entry_from_row(row: &Row) -> Result<DangerousAction, RusqliteError> {
    Ok(DangerousAction {
        id: row.get(0)?,
        people_id: row.get(1)?,
        token: row.get(2)?,
        dangerous_action_kind_id: row.get(3)?,
        contact_kind_id: row.get(4)?,
        contact_content: row.get(5)?,
        deleted_at: row.get(6)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS dangerous_actions (
            id INTEGER PRIMARY KEY,
            people_id INTEGER,
            token INTEGER NOT NULL,
            dangerous_action_kind_id INTEGER NOT NULL,
            contact_kind_id INTEGER NOT NULL,
            contact_content TEXT NOT NULL UNIQUE,
            deleted_at INTEGER,
            UNIQUE (dangerous_action_kind_id, contact_kind_id, contact_content)
        )",
        (),
    );

    if let Err(e) = results {
        return Err("dangerous_actions table error: \n".to_string() + &e.to_string());
    }

    Ok(())
}

pub struct CreateParams<'a> {
    pub id: u64,
    pub people_id: Option<u64>,
    pub token: u64,
    pub dangerous_action_kind_id: u64,
    pub contact_kind_id: u64,
    pub contact_content: &'a str,
}

pub fn create(conn: &mut Connection, params: &CreateParams) -> Result<DangerousAction, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO dangerous_actions
            (id, people_id, token, dangerous_action_kind_id, contact_kind_id, contact_content)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("failed to create a dangerous action".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (
            params.id,
            params.people_id,
            params.token,
            params.dangerous_action_kind_id,
            params.contact_kind_id,
            params.contact_content,
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

    Err("failed to create dangerous action".to_string())
}

pub fn read(conn: &mut Connection, id: u64) -> Result<Option<DangerousAction>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            dangerous_actions
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
    pub dangerous_action_kind_id: u64,
    pub contact_kind_id: u64,
    pub contact_content: &'a str,
}

pub fn read_by_kind_id_and_content(
    conn: &mut Connection,
    params: &ReadByKindIdAndContentParams,
) -> Result<Option<DangerousAction>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            dangerous_actions
        WHERE
            deleted_at IS NULL
            AND
            dangerous_action_kind_id = ?1
            AND
            contact_kind_id = ?2
            AND
            contact_content = ?3
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("failed to read a contact by id".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (
            params.dangerous_action_kind_id,
            params.contact_kind_id,
            params.contact_content,
        ),
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

// RATELIMIT DANGEROUS ACTIONS