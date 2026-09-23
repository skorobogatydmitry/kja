//! Database requests handler

use std::sync::{LazyLock, Mutex};

use refinery::embed_migrations;
use rusqlite::Connection;

use crate::record::{KanjiReading, Word};

static DATABASE_FILE_PATH: &str = concat!(env!("CARGO_PKG_NAME"), ".sqlite");

pub(super) static DATABASE: LazyLock<Mutex<Database>> = LazyLock::new(|| {
    let mut connection = Connection::open(DATABASE_FILE_PATH).expect("cannot connect to database");
    migrations::runner()
        .run(&mut connection)
        .expect("database migrations failed");
    Mutex::new(Database { connection })
});

embed_migrations!();

pub(super) struct Database {
    connection: Connection,
}

impl Database {
    pub(super) fn next_by_kanji(&mut self, kanji: char, current_word: Word) -> Word {
        todo!("get another word with the same kanji, != current_word")
    }

    pub(super) fn next_by_reading(&mut self, reading: &KanjiReading, current_word: Word) -> Word {
        todo!("get another word with the same kanji reading, != current_word")
    }
}

// fn main() -> rusqlite::Result<()> {
//     let mut conn = Connection::open("data.db")?;
//     migrations::runner().run(&mut conn).unwrap();

//     conn.execute("INSERT INTO users (name) VALUES (?1)", [&"Alice"])?;

//     let mut stmt = conn.prepare("SELECT id, name FROM users")?;
//     let rows = stmt.query_map([], |row| {
//         Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
//     })?;
//     for r in rows {
//         println!("{:?}", r?);
//     }
//     Ok(())
// }
