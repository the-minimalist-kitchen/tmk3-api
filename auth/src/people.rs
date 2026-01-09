use crate::utils::{
    create_token_u64, deserialize_session, get_connection, get_snowflake_id, get_timestamp,
    hash_password, serialize_session_token, set_connection, validate_password,
};
use crate::Auth;
use sqlite_interfaces::contact_kinds::read_by_kind as read_contact_kind_by_kind;
use sqlite_interfaces::contacts::{
    create as create_contact, read_by_kind_id_and_content as read_contact_by_kind_id_and_content,
    CreateParams as CreateContactParams,
    ReadByKindIdAndContentParams as ReadContactByKindIdAndContentParams,
};
use sqlite_interfaces::dangerous_action_kinds::read_by_kind as read_action_by_kind;
use sqlite_interfaces::dangerous_actions::{
    create as create_action, read as read_action, CreateParams as CreateDangerousActionParams,
};
use sqlite_interfaces::people::{create as create_person, read as read_person};
use sqlite_interfaces::public_sessions::{
    delete_all as delete_all_public_sessions, read as read_public_session,
    DeleteAllParams as DeleteAllPublicSessionsParams,
};
use sqlite_interfaces::sessions::{delete as delete_session, DeleteParams as DeleteSessionParams};
use type_flyweight::contacts::Contact;
use type_flyweight::sessions::{PublicSession, SessionToken};

pub struct RequestAccountParams<'a> {
    contact_kind: &'a str,
    contact_content: &'a str,
}

pub struct RegisterAccountParams<'a> {
    session_str: &'a str,
    password: &'a str,
}

pub struct LoginParams<'a> {
    contact_kind: &'a str,
    contact_content: &'a str,
    password: &'a str,
}

impl Auth {
    // STATES
    // Contact already exists
    // Success
    //
    pub fn request_account(&self, params: &RequestAccountParams) -> Result<String, String> {
        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };

        let contact_kind = match read_contact_kind_by_kind(&mut conn, params.contact_kind) {
            Ok(maybe_kind) => match maybe_kind {
                Some(kind) => kind,
                _ => return Err("no contact kind found!".to_string()),
            },
            Err(e) => return Err(e),
        };

        // get contact by kind and content
        // if exists bail
        let contact_maybe = match read_contact_by_kind_id_and_content(
            &mut conn,
            &ReadContactByKindIdAndContentParams {
                contact_kind_id: contact_kind.id,
                content: params.contact_content,
            },
        ) {
            Ok(maybe_contact) => maybe_contact,
            Err(e) => return Err(e),
        };
        if let Some(_contact) = contact_maybe {
            return Err("Error might already exist!".to_string());
        }

        let action_kind = match read_action_by_kind(&mut conn, "request_account") {
            Ok(maybe_kind) => match maybe_kind {
                Some(kind) => kind,
                _ => return Err("no action kind found".to_string()),
            },
            Err(e) => return Err(e),
        };

        // create action
        let id = match get_snowflake_id(&self.params.snowprints) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        let token = create_token_u64();

        let dangerous_action = match create_action(
            &mut conn,
            &CreateDangerousActionParams {
                id,
                people_id: None,
                token,
                dangerous_action_kind_id: action_kind.id,
                contact_kind_id: contact_kind.id,
                contact_content: &params.contact_content,
            },
        ) {
            Ok(action) => action,
            Err(e) => return Err(e),
        };

        let _ = set_connection(&self.params.connection_pool, conn);

        Ok(serialize_session_token(&SessionToken {
            id: dangerous_action.id,
            people_id: None,
            token: dangerous_action.token,
        }))
    }

    // Account already registerd
    pub fn register_account(&self, params: &RegisterAccountParams) -> Result<Contact, String> {
        let session_token = match deserialize_session(params.session_str) {
            Ok(token) => token,
            Err(e) => return Err(e),
        };

        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };

        // CREATE A READ AND DELETE DANGEROUS ACTION
        // read and delete action
        let action = match read_action(&mut conn, session_token.id) {
            Ok(action_maybe) => match action_maybe {
                Some(action) => action,
                _ => return Err("action does not exist".to_string()),
            },
            Err(e) => return Err(e),
        };

        if action.token != session_token.token {
            return Err("compromised session token".to_string());
        }

        // check if contact exists
        let contact_maybe = match read_contact_by_kind_id_and_content(
            &mut conn,
            &ReadContactByKindIdAndContentParams {
                contact_kind_id: action.contact_kind_id,
                content: &action.contact_content,
            },
        ) {
            Ok(maybe_contact) => maybe_contact,
            Err(e) => return Err(e),
        };
        if let Some(_contact) = contact_maybe {
            return Err("Error might already exist!".to_string());
        }

        // create person
        let people_id = match get_snowflake_id(&self.params.snowprints) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        let hashed_password = match hash_password(params.password) {
            Ok(hp) => hp,
            Err(e) => return Err(e),
        };

        let person = match create_person(&mut conn, people_id, &hashed_password) {
            Ok(person) => person,
            Err(e) => return Err(e),
        };

        // create contact
        let id = match get_snowflake_id(&self.params.snowprints) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        create_contact(
            &mut conn,
            &CreateContactParams {
                id,
                people_id: person.id,
                contact_kind_id: action.contact_kind_id,
                content: &action.contact_content,
            },
        )
    }

    pub fn login(&mut self, params: &LoginParams) -> Result<String, String> {
        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };

        let contact_kind = match read_contact_kind_by_kind(&mut conn, params.contact_kind) {
            Ok(maybe_kind) => match maybe_kind {
                Some(kind) => kind,
                _ => return Err("no contact kind found!".to_string()),
            },
            Err(e) => return Err(e),
        };

        let contact = match read_contact_by_kind_id_and_content(
            &mut conn,
            &ReadContactByKindIdAndContentParams {
                contact_kind_id: contact_kind.id,
                content: params.contact_content,
            },
        ) {
            Ok(maybe_contact) => match maybe_contact {
                Some(kind) => kind,
                _ => return Err("no contact found!".to_string()),
            },
            Err(e) => return Err(e),
        };

        let person = match read_person(&mut conn, contact.people_id) {
            Ok(person_maybe) => match person_maybe {
                Some(person) => person,
                _ => return Err("no person found!".to_string()),
            },
            Err(e) => return Err(e),
        };

        if !validate_password(&person.password_hash_results, params.password) {
            return Err("failed to validate password".to_string());
        }

        // call private create session
        self.create_session(Some(person.id))
    }

    pub fn logout(&self, session_str: &str) -> Result<Vec<PublicSession>, String> {
        let session_token = match deserialize_session(session_str) {
            Ok(st) => st,
            Err(e) => return Err(e),
        };

        // get public session
        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };

        // read public session
        let public_session = match read_public_session(&mut conn, &session_token) {
            Ok(public_session_maybe) => match public_session_maybe {
                Some(ps) => ps,
                _ => return Err("no public session found!".to_string()),
            },
            Err(e) => return Err(e),
        };

        // delete session
        let current_timestamp = match get_timestamp(&self.params.snowprints) {
            Ok(ts) => ts,
            Err(e) => return Err(e),
        };

        let session = match delete_session(
            &mut conn,
            &DeleteSessionParams {
                id: public_session.session_id,
                current_timestamp,
            },
        ) {
            Ok(session_maybe) => match session_maybe {
                Some(ps) => ps,
                _ => return Err("no session found!".to_string()),
            },
            Err(e) => return Err(e),
        };

        // delete all corresponding public sessions
        delete_all_public_sessions(
            &mut conn,
            &DeleteAllPublicSessionsParams {
                session_id: session.id,
                current_timestamp,
            },
        )
    }

    // get all active sessions

    // remove specific session

    // request_account_deletion

    // confirm_account_deletion
}
