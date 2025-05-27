use rusqlite::{fallible_streaming_iterator::FallibleStreamingIterator, Connection, Error};
use uuid::Uuid;

pub struct SessionToken {
    pub token: String
}

pub(crate) struct SessionDB {
    conn: rusqlite::Connection
}

impl SessionDB {

    pub fn new() -> Result<SessionDB, Error> {
        //let c = Connection::open_in_memory()?;
        let c = Connection::open("./session.db")?;
        {
            let mut stmt = c.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
            let rows = stmt.query(())?;
            if let Ok(0) = rows.count() {
                c.execute("CREATE TABLE sessions (token TEXT PRIMARY KEY, username TEXT NOT NULL)", ())?;
            }
        }
        Ok(SessionDB { conn: c })
    }

    pub fn new_session(&self, username: String) -> Result<SessionToken, Error> {
        println!("New session for user: {}", username);
        let session = Uuid::new_v4().to_string();
        self.conn.execute("INSERT INTO sessions (token, username) VALUES (?1, ?2)", (session.clone(), username))?;
        return Ok(SessionToken{token: session});
    }

    pub fn get_user(&self, token: SessionToken) -> Option<String> {
        println!("Getting user for session token:{}", token.token);
        if let Ok(mut stmt) = self.conn.prepare("SELECT username FROM sessions WHERE token = :token") {
            let res = stmt.query_row(rusqlite::named_params! {":token": token.token.clone()}, |row| Ok(row.get(0)?));
            println!("Got result:{:?}", res);
            match res {
                Ok(username) => Some(username),
                _ => None
            }
        }
        else {
            println!("Statement failed");
            None
        }
    }
}