use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct ConnectionPool {
    db_path: PathBuf,
    max_connection_count: usize,
    connections: Mutex<Vec<Connection>>,
}

impl ConnectionPool {
    pub fn from(db_path: &PathBuf, max_connection_count: usize) -> Result<ConnectionPool, String> {
        if max_connection_count == 0 {
            return Err("max connections cannot be 0".to_string());
        }

        Ok(ConnectionPool {
            db_path: db_path.clone(),
            max_connection_count: max_connection_count,
            connections: Mutex::new(Vec::new()),
        })
    }

    pub fn get_connection(&mut self) -> Result<Connection, String> {
        let mut connections = match self.connections.lock() {
            Ok(connections) => connections,
            Err(e) => return Err(e.to_string()),
        };

        if let Some(conn) = connections.pop() {
            return Ok(conn);
        }

        match Connection::open(&self.db_path) {
            Ok(cn) => Ok(cn),
            Err(e) => return Err(e.to_string()),
        }
    }

    pub fn set_connection(&mut self, conn: Connection) -> Result<(), String> {
        let mut connections = match self.connections.lock() {
            Ok(connections) => connections,
            Err(e) => return Err(e.to_string()),
        };

        if connections.len() < self.max_connection_count {
            connections.push(conn);
        }

        Ok(())
    }
}
