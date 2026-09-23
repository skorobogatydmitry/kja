//! Database requests handler

use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

use anyhow::Context;
use refinery::embed_migrations;
use rusqlite::Connection;

use crate::record::{KanjiReading, Word};

static DATABASE_FILE_PATH: &str = concat!(env!("CARGO_PKG_NAME"), ".sqlite");

pub(super) static DATABASE: LazyLock<Mutex<Database>> = LazyLock::new(|| {
    let mut connection = Connection::open(DATABASE_FILE_PATH).expect("cannot connect to database");
    migrations::runner()
        .run(&mut connection)
        .expect("database migrations failed");
    Mutex::new(Database {
        connection,
        words_tail: HashSet::new(),
    })
});

embed_migrations!();

pub(super) struct Database {
    connection: Connection,
    // list of already used words
    words_tail: HashSet<Word>,
}

impl Database {
    /// get a random word
    pub(super) fn random_word(&mut self) -> Word {
        let word_id = self
            .connection
            .prepare("select word_id from word_component order by random() limit 1")
            .expect("cannot get random word")
            .query_one([], |r| r.get::<_, u32>(0))
            .expect("cannot parse random word id");
        let mut statement = self.connection.prepare("
            select kr.kanji, kr.reading
                from word_component as wc left join kanji_reading as kr on wc.kanji_reading_id = kr.id
                where word_id = ?1
                order by wc.word_position")
            .expect("error on making statement for random word");
        let readings = statement
            .query_map([word_id], |row| {
                Ok(KanjiReading {
                    kanji: row.get::<_, String>(0)?.chars().next().unwrap(),
                    furigana: row.get::<_, String>(1)?,
                })
            })
            .expect("error selecting random word from database")
            .collect::<Result<Vec<_>, _>>()
            .expect("error selecting random word from database");

        let translations = self
            .connection
            .prepare("select translation from word_translation where word_id = ?1")
            .expect("cannot get translations for the word")
            .query_map([word_id], |row| Ok(row.get::<_, String>(0)?))
            .expect("error selecting random word from database")
            .collect::<Result<Vec<_>, _>>()
            .expect("error selecting random word from database");

        let word = Word {
            readings,
            translations,
        };
        self.words_tail.insert(word.clone());

        word
    }

    /// get next word with the same kanji (maybe different reading)
    pub(super) fn next_by_kanji(&mut self, kanji: char) -> Word {
        todo!("get another word with the same kanji, != current_word")
    }

    /// get next word with the same kanji and its reading (maybe different reading)
    pub(super) fn next_by_reading(&mut self, reading: &KanjiReading) -> Word {
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
