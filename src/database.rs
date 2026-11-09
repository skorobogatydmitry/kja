//! Database requests handler

use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex, MutexGuard},
};

use log::warn;
use rand::seq::IteratorRandom;
use refinery::embed_migrations;
use rusqlite::Connection;

use crate::word::{KanjiReading, Word};

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
    // list of already used word ids
    words_tail: HashSet<u32>,
}

impl Database {
    /// add a new word to the database
    pub(super) fn add_word(new_word: Word) -> anyhow::Result<u32> {
        let mut db = Database::get();

        let reading_ids = new_word
            .readings
            .into_iter()
            .enumerate()
            .map(|(idx, r)| (idx + 1, db.upsert_reading(r)))
            .rev()
            .collect::<Vec<(usize, u32)>>();

        // TODO: check for dublicates

        let new_word_id = db
            .connection
            .query_one("select max(word_id) + 1 from word_component", [], |r| {
                r.get::<_, u32>(0)
            })
            .unwrap();

        let mut statement = db.connection
            .prepare("insert into word_component(word_id,word_position,kanji_reading_id) values (?1, ?2, ?3)")
            .unwrap();

        reading_ids.into_iter().for_each(|(pos, reading_id)| {
            statement
                .execute([new_word_id, pos as u32, reading_id])
                .unwrap();
        });

        Ok(new_word_id)
    }

    /// insert or get its id
    fn upsert_reading(&mut self, reading: KanjiReading) -> u32 {
        self
            .connection
            .query_one(
                "insert into kanji_reading (kanji, reading) values (?1, ?2) on conflict do update set reading = excluded.reading returning id",
                [
                    reading.kanji.to_string().as_str(),
                    reading.furigana.as_str(),
                ],
                |r| r.get::<_, u32>(0),
            )
            .expect("error upserting a new reading")
    }

    /// get a random word
    pub(super) fn random_word() -> Word {
        let mut db = Self::get();
        let word_id = db
            .connection
            .query_one(
                "select word_id from word_component order by random() limit 1",
                [],
                |r| r.get::<_, u32>(0),
            )
            .expect("cannot parse random word id");

        db.words_tail.insert(word_id);
        db.fetch_word_by_id(word_id)
    }

    /// get next word with the same kanji (maybe different reading)
    pub(super) fn next_by_kanji(kanji: char) -> Word {
        // TODO: exclude current word
        let mut db = Self::get();
        let available_word_ids = db
            .connection
            .prepare("select word_id from word_component where kanji_reading_id in (select id from kanji_reading where kanji = ?1)")
            .expect("cannot get word ids for kanji")
            .query_map([kanji.to_string()], |r| r.get::<_, u32>(0))
            .expect("cannot parse word ids")
            .collect::<Result<HashSet<_>, _>>()
            .expect("cannot parse word ids");

        let unused_words = available_word_ids
            .difference(&db.words_tail)
            .cloned()
            .collect::<Vec<u32>>();

        let word_id = *if unused_words.is_empty() {
            warn!("no new words with kanji {}, using random one", kanji);
            available_word_ids.iter().choose(&mut rand::rng())
        } else {
            unused_words.iter().choose(&mut rand::rng())
        }
        .expect("no available words for the kanji"); // TODO: always works, as soon as the current word is not excluded

        db.words_tail.insert(word_id);
        db.fetch_word_by_id(word_id)
    }

    /// get next word with the same kanji and its reading (maybe different reading)
    pub(super) fn next_by_reading(reading: &KanjiReading) -> Word {
        // TODO: exclude current word
        let mut db = Self::get();
        let available_word_ids = db
            .connection
            .prepare("select word_id from word_component where kanji_reading_id in (select id from kanji_reading where kanji = ?1 and reading = ?2)")
            .expect("cannot get word ids for reading")
            .query_map([reading.kanji.to_string(), reading.furigana.clone()], |r| {
                r.get::<_, u32>(0)
            })
            .expect("cannot parse word ids")
            .collect::<Result<HashSet<_>, _>>()
            .expect("cannot parse word ids");

        let unused_words = available_word_ids
            .difference(&db.words_tail)
            .cloned()
            .collect::<Vec<u32>>();

        let word_id = *if unused_words.is_empty() {
            warn!(
                "no new words with reading {}, using random one",
                reading.furigana
            );
            available_word_ids.iter().choose(&mut rand::rng())
        } else {
            unused_words.iter().choose(&mut rand::rng())
        }
        .expect("no available words for the reading"); // TODO: always works, as soon as the current word is not excluded

        db.words_tail.insert(word_id);
        db.fetch_word_by_id(word_id)
    }

    /// get word by ID from the database
    fn fetch_word_by_id(&mut self, word_id: u32) -> Word {
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
            .query_map([word_id], |row| row.get::<_, String>(0))
            .expect("error selecting random word from database")
            .collect::<Result<Vec<_>, _>>()
            .expect("error selecting random word from database");
        Word {
            readings,
            translations,
        }
    }

    fn get() -> MutexGuard<'static, Database> {
        DATABASE.lock().unwrap()
    }
}
