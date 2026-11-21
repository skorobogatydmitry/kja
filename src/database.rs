//! Database requests handler

use std::{
    collections::HashSet,
    fs,
    sync::{LazyLock, Mutex, MutexGuard, OnceLock},
};

use anyhow::Context;
use log::warn;
use rand::seq::IteratorRandom;
use refinery::embed_migrations;
use rusqlite::{Connection, params};

use crate::{
    message,
    word::{KanjiReading, Word},
};

pub static DATABASE_FILE_PATH: OnceLock<String> = OnceLock::new();
pub const DEFAULT_DATABASE_FILE_PATH: &str = concat!(env!("CARGO_PKG_NAME"), ".sqlite");

static DATABASE: LazyLock<Mutex<Database>> = LazyLock::new(|| {
    let database_file_path = DATABASE_FILE_PATH
        .get()
        .cloned()
        .unwrap_or(DEFAULT_DATABASE_FILE_PATH.to_string());
    match fs::exists(&database_file_path) {
        Ok(true) => log::info!("Database will be loaded from {database_file_path}"),
        Ok(false) => panic!("Database file not found at {database_file_path}"),
        Err(e) => panic!("Cannot check database at {database_file_path}: {e}"),
    }
    let mut connection = Connection::open(database_file_path).expect("cannot connect to database");
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
            .map(|(idx, r)| db.upsert_reading(r).map(|r_id| (idx + 1, r_id)))
            .collect::<anyhow::Result<Vec<(usize, u32)>>>()?;

        // TODO: check for the same word

        let new_word_id =
            db.connection
                .query_one("insert into word default values returning id", [], |r| {
                    r.get::<_, u32>(0)
                })?;

        let mut statement = db.connection
            .prepare("insert into word_component(word_id,word_position,kanji_reading_id) values (?1, ?2, ?3)")?;

        reading_ids.into_iter().try_for_each(|(pos, reading_id)| {
            statement
                .execute([new_word_id, pos as u32, reading_id])
                .map(drop)
        })?;

        let mut translation_insert_statement = db
            .connection
            .prepare("insert into word_translation(word_id,translation) values (?1, ?2)")?;

        new_word.translations.into_iter().try_for_each(|t| {
            translation_insert_statement
                .execute(params![new_word_id, t])
                .map(drop)
        })?;

        Ok(new_word_id)
    }

    /// insert or get its id
    fn upsert_reading(&mut self, reading: KanjiReading) -> anyhow::Result<u32> {
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
            .context("error upserting a new reading")
    }

    /// get a random word
    pub(super) fn random_word() -> Word {
        let mut db = Self::get();
        let word_id = db
            .connection
            .query_one("select id from word order by random() limit 1", [], |r| {
                r.get::<_, u32>(0)
            })
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
            message::show(format!(
                "no new words with kanji {}, using random one",
                kanji
            ));
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
