use crate::utils::{
    create_token_u64, deserialize_session, get_connection, get_snowflake_id, get_timestamp,
    hash_password, serialize_session_token, set_connection, validate_password,
};
use crate::Auth;
use sqlite_interfaces::contact_kinds::read_by_kind as read_contact_kind_by_kind;
use sqlite_interfaces::contacts;
use sqlite_interfaces::contacts::{
    create as create_contact, read_by_kind_id_and_content, CreateParams as CreateContactParams,
    ReadByKindIdAndContentParams as ReadContactByKindIdAndContentParams,
};
use sqlite_interfaces::dangerous_actions;

use sqlite_interfaces::dangerous_action_kinds::read_by_kind as read_action_by_kind;
use sqlite_interfaces::people;
use sqlite_interfaces::public_sessions::{
    delete_all as delete_all_public_sessions, read as read_public_session,
    DeleteAllParams as DeleteAllPublicSessionsParams,
};
use sqlite_interfaces::sessions::{delete as delete_session, DeleteParams as DeleteSessionParams};
use type_flyweight::contacts::Contact;
use type_flyweight::sessions::{PublicSession, SessionToken};

pub struct RequestContactParams<'a> {
    // session_str
    people_id: u64,
    contact_kind: &'a str,
    contact_content: &'a str,
}

pub struct RegisterContactParams<'a> {
    session_str: &'a str,
}

impl Auth {
    // STATES
    // Contact already exists
    // Success
    //
    pub fn request_contact(&self, params: &RequestContactParams) -> Result<String, String> {
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
        let contact_maybe = match contacts::read_by_kind_id_and_content(
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

        let id = match get_snowflake_id(&self.params.snowprints) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        let token = create_token_u64();

        let dangerous_action = match dangerous_actions::create(
            &mut conn,
            &dangerous_actions::CreateParams {
                id,
                people_id: Some(params.people_id),
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
    pub fn register_contact(&self, params: &RegisterContactParams) -> Result<Contact, String> {
        let session_token = match deserialize_session(params.session_str) {
            Ok(token) => token,
            Err(e) => return Err(e),
        };

        let mut conn = match get_connection(&self.params.connection_pool) {
            Ok(conn) => conn,
            Err(e) => return Err(e),
        };

        // get person
        let person = match people::read(&mut conn, session_token.id) {
            Ok(person_maybe) => match person_maybe {
                Some(person) => person,
                _ => return Err("did not find a person".to_string()),
            },
            Err(e) => return Err(e),
        };

        // CREATE A READ AND DELETE DANGEROUS ACTION
        // read and delete action
        let action = match dangerous_actions::read(&mut conn, session_token.id) {
            Ok(action_maybe) => match action_maybe {
                Some(action) => action,
                _ => return Err("action does not exist".to_string()),
            },
            Err(e) => return Err(e),
        };

        // verify token
        if action.token != session_token.token {
            return Err("compromised session token".to_string());
        }

        // check if contact exists
        let contact_maybe = match contacts::read_by_kind_id_and_content(
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

        // create contact
        let contact_id = match get_snowflake_id(&self.params.snowprints) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        // validate password .. or .... ??
        // if !validate_password(params.password) {
        //     return Err("password was not valid".to_string())
        // };

        // let person = match contacts::create(contacts::CreateParams {
        //             pub id: u64,
        //             pub people_id: u64,
        //             pub contact_kind_id: u64,
        //             pub content: &'a str,
        // } {
        //     Ok(person) => person,
        //     Err(e) => return Err(e),
        // };

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
}
