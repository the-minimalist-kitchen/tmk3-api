use rusqlite::{Connection, Result};
use sqlite_interfaces::dangerous_actions;
use sqlite_interfaces::dangerous_actions::{CreateParams, ReadByKindIdAndContentParams};

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = dangerous_actions::create_table(&mut conn) {
        assert!(false, "failed to create dangerous_actions table");
    }

    // create
    let contact = match dangerous_actions::create(
        &mut conn,
        &CreateParams {
            id: 1,
            people_id: Some(2),
            token: 12341234,
            dangerous_action_kind_id: 6,
            contact_kind_id: 3,
            contact_content: "email@email.email",
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let contact_read_by_id = match dangerous_actions::read(&mut conn, 1) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(contact.clone()) == contact_read_by_id);

    // read by kind and content
    let dangerous_action_read_by_kind = match dangerous_actions::read_by_kind_id_and_content(
        &mut conn,
        &ReadByKindIdAndContentParams {
            dangerous_action_kind_id: 6,
            contact_kind_id: 3,
            contact_content: "email@email.email",
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(contact) == dangerous_action_read_by_kind);

    Ok(())
}
