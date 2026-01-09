use rusqlite::{Connection, Result};
use sqlite_interfaces::totp;
use sqlite_interfaces::totp::CreateParams;

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = totp::create_table(&mut conn) {
        assert!(false, "failed to create totp table");
    }

    // create
    let totp = match totp::create(
        &mut conn,
        CreateParams {
            id: 1,
            people_id: 64,
            secret_key: "walk your heart into the sea",
            algorithm: None,
            period: None,
            digits: None,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let totp_read_by_id = match totp::read(&mut conn, 1) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(totp) == totp_read_by_id);

    Ok(())
}
