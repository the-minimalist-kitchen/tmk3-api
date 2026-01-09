use rusqlite::{Connection, Error as RusqliteError, Result, Row};
use type_flyweight::people::Person;

fn get_entry_from_row(row: &Row) -> Result<Person, RusqliteError> {
    Ok(Person {
        id: row.get(0)?,
        password_hash_results: row.get(1)?,
        deleted_at: row.get(2)?,
    })
}

pub fn create_table(conn: &mut Connection) -> Result<(), String> {
    let results = conn.execute(
        "CREATE TABLE IF NOT EXISTS people (
            id INTEGER PRIMARY KEY,
            password_hash_results TEXT NOT NULL,
            deleted_at INTEGER
        )",
        (),
    );

    if let Err(e) = results {
        return Err("people table error: \n".to_string() + &e.to_string());
    }

    Ok(())
}

pub fn create(
    conn: &mut Connection,
    id: u64,
    password_hash_results: &str,
) -> Result<Person, String> {
    let mut stmt = match conn.prepare(
        "
        INSERT INTO people
            (id, password_hash_results)
        VALUES
            (?1, ?2)
        RETURNING
            *
    ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not create person".to_string()),
    };

    let mut entry_iter = match stmt.query_map((id, password_hash_results), get_entry_from_row) {
        Ok(entry_iter) => entry_iter,
        Err(e) => return Err(e.to_string()),
    };

    if let Some(entry_maybe) = entry_iter.next() {
        if let Ok(entry) = entry_maybe {
            return Ok(entry);
        }
    }

    Err("failed to create person (people)".to_string())
}

pub fn read(conn: &mut Connection, id: u64) -> Result<Option<Person>, String> {
    let mut stmt = match conn.prepare(
        "
        SELECT
            *
        FROM
            people
        WHERE
            id = ?1
        ",
    ) {
        Ok(stmt) => stmt,
        _ => return Err("cound not read person".to_string()),
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
