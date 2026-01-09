use rusqlite::{Connection, Result};
use sqlite_interfaces::public_sessions;
use sqlite_interfaces::public_sessions::{
    CreateParams, RateLimitParams, ReadAllByIdParams, ReadAllByPeopleIdParams,
};
use type_flyweight::sessions::{PublicSession, SessionToken};

#[test]
fn crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open_in_memory()?;

    if let Err(_e) = public_sessions::create_table(&mut conn) {
        assert!(false, "failed to create public_session table");
    }

    // create
    let public_session = match public_sessions::create(
        &mut conn,
        &CreateParams {
            id: 16,
            people_id: Some(64),
            token: 7654,
            session_id: 19,
            current_timestamp: 10,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    // read by id
    let public_session_read_by_id = match public_sessions::read(
        &mut conn,
        &SessionToken {
            id: 16,
            people_id: Some(64),
            token: 7654,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(Some(public_session) == public_session_read_by_id);

    let public_session_read_all_by_session_id = match public_sessions::read_all_by_session_id(
        &mut conn,
        &ReadAllByIdParams {
            id: 19,
            limit: 5,
            offset: 0,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };
    // read by kind and content
    let public_session_read_all_by_people_id = match public_sessions::read_all_by_people_id(
        &mut conn,
        &ReadAllByPeopleIdParams {
            people_id: Some(64),
            offset: 0,
            limit: 5,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    assert!(public_session_read_all_by_people_id == public_session_read_all_by_session_id);
    assert!(Vec::<PublicSession>::new() != public_session_read_all_by_people_id);
    assert!(Vec::<PublicSession>::new() != public_session_read_all_by_session_id);

    // rate_limit_session
    let entry_maybe = match public_sessions::rate_limit_session(
        &mut conn,
        &RateLimitParams {
            id: 16,
            people_id: Some(64),
            token: 7654,
            current_timestamp: 20,
            window_limit: 10,
            window_length_ms: 5,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    if let None = entry_maybe {
        assert!(false, "should have found a session to rate limit")
    }

    if let Some((should_rate_limit, _entry)) = &entry_maybe {
        assert!(!should_rate_limit, "should not have rate limited session");
    };

    let entry_maybe2 = match public_sessions::rate_limit_session(
        &mut conn,
        &RateLimitParams {
            id: 16,
            people_id: Some(64),
            token: 7654,
            current_timestamp: 22,
            window_limit: 10,
            window_length_ms: 5,
        },
    ) {
        Ok(ck) => ck,
        Err(e) => return Err(e.into()),
    };

    if let None = entry_maybe2 {
        assert!(false, "should have found a session to rate limit")
    }

    if let Some((should_rate_limit, _entry)) = &entry_maybe2 {
        assert!(
            !should_rate_limit,
            "should not have rate limited session on second try"
        )
    };

    assert!(entry_maybe != entry_maybe2);

    Ok(())
}
