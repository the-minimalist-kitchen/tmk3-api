use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use type_flyweight::totp::Totp;

fn get_entry_from_row(row: &Row) -> Result<Totp, RusqliteError> {
    Ok(Totp {
        id: row.get(0)?,
        people_id: row.get(1)?,
        secret_key: row.get(2)?,
        algorithm: row.get(3)?,
        period: row.get(4)?,
        digits: row.get(5)?,
        deleted_at: row.get(6)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS totp (
            id INTEGER PRIMARY KEY,
            people_id INTEGER NOT NULL,
            secret_key TEXT NOT NULL,
            algorithm INTEGER,
            period INTEGER,
            digits INTEGER,
            deleted_at INTEGER
        )",
        (),
    );

    if let Err(e) = results {
        return Err("totp table error: \n".to_string() + &e.to_string());
    }

    Ok(())
}

pub struct CreateParams<'a> {
    pub id: u64,
    pub people_id: u64,
    pub secret_key: &'a str,
    pub algorithm: Option<u64>,
    pub period: Option<u64>,
    pub digits: Option<u64>,
}

pub fn create(conn: &mut Connection, params: CreateParams) -> Result<Totp, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO totp
            (id, people_id, secret_key, algorithm, period, digits)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not create totp".to_string()),
    };

    let mut entry_iter = match stmt.query_map(
        (
            params.id,
            params.people_id,
            params.secret_key,
            params.algorithm,
            params.period,
            params.digits,
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

    Err("failed to create TOTP".to_string())
}

pub fn read(conn: &mut Connection, id: u64) -> Result<Option<Totp>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            totp
        WHERE
            id = ?1
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not read totp".to_string()),
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
