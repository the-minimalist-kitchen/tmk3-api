use rusqlite::{Connection, Result};
use sqlite_interfaces::sessions;
use sqlite_interfaces::sessions::{CreateParams, ReadAllByPeopleIdParams};
use type_flyweight::sessions::Session;

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = sessions::create_table(&mut conn) {
        assert!(false, "failed to create sessions table");
    }

    // create
    let session = match sessions::create(
        &mut conn,
        &CreateParams {
            id: 16,
            people_id: Some(42),
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let session_read_by_id = match sessions::read(&mut conn, 16) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(session.clone()) == session_read_by_id);

    // read by kind and content
    let session_read_all_by_people_id = match sessions::read_all_by_people_id(
        &mut conn,
        &ReadAllByPeopleIdParams {
            people_id: Some(42),
            offset: 0,
            limit: 1,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Vec::from([session]) == session_read_all_by_people_id);
    assert!(Vec::<Session>::new() != session_read_all_by_people_id);

    Ok(())
}
