use rusqlite::{Connection, Result};
use sqlite_interfaces::contact_kinds;

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = contact_kinds::create_table(&mut conn) {
        assert!(false, "failed to create contact_kinds table");
    }

    // create
    let contact_kind = match contact_kinds::create(&mut conn, 1, "email") {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let contact_kind_read_by_id = match contact_kinds::read(&mut conn, 1) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(contact_kind.clone()) == contact_kind_read_by_id);

    // read by kind
    let contact_kind_read_by_kind = match contact_kinds::read_by_kind(&mut conn, "email") {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(contact_kind) == contact_kind_read_by_kind);

    Ok(())
}
