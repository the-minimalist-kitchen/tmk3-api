use rusqlite::{Connection, Result};
use sqlite_interfaces::people;

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = people::create_table(&mut conn) {
        assert!(false, "failed to create people table");
    }
    // create
    let person = match people::create(
        &mut conn,
        1,
        "fold a piece of paper into something that you love",
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let person_read_by_id = match people::read(&mut conn, 1) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(person) == person_read_by_id);

    Ok(())
}
