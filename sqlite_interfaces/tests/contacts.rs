use rusqlite::{Connection, Result};
use sqlite_interfaces::contacts;
use sqlite_interfaces::contacts::{CreateParams, ReadByKindIdAndContentParams};

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = contacts::create_table(&mut conn) {
        assert!(false, "failed to create contacts table");
    }

    // create
    let contact = match contacts::create(
        &mut conn,
        &CreateParams {
            id: 1,
            people_id: 2,
            contact_kind_id: 3,
            content: "email@email.email",
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let contact_read_by_id = match contacts::read(&mut conn, 1) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(contact.clone()) == contact_read_by_id);

    // read by kind and content
    let contact_read_by_kind = match contacts::read_by_kind_id_and_content(
        &mut conn,
        &ReadByKindIdAndContentParams {
            contact_kind_id: 3,
            content: "email@email.email",
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(contact) == contact_read_by_kind);

    Ok(())
}
