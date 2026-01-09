use rusqlite::{Connection, Result};
use sqlite_interfaces::dangerous_action_kinds;

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = dangerous_action_kinds::create_table(&mut conn) {
        assert!(false, "failed to create dangerous_action_kinds table");
    }

    // create
    let dangerous_action_kind = match dangerous_action_kinds::create(&mut conn, 1, "email", 47) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let dangerous_action_kind_read_by_id = match dangerous_action_kinds::read(&mut conn, 1) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(dangerous_action_kind.clone()) == dangerous_action_kind_read_by_id);

    // read by kind
    let dangerous_action_kind_read_by_kind =
        match dangerous_action_kinds::read_by_kind(&mut conn, "email") {
            Ok(ck) => ck,
            Err(e) => return Err(e.into()),
        };

    assert!(Some(dangerous_action_kind) == dangerous_action_kind_read_by_kind);

    Ok(())
}
