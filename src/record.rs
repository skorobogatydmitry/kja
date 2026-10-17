//! word record representation
//!

use std::vec::Drain;

use crate::database::DATABASE;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct Word {
    // TODO: words like きょう　/ 今日
    pub readings: Vec<KanjiReading>,
    pub translations: Vec<String>,
}

impl Word {
    /// make a word from list of kanjis and readings
    pub fn from_input(
        kanjis: Drain<char>,
        furigana: Drain<String>,
        translations: Drain<String>,
    ) -> Self {
        let readings = kanjis
            .zip(furigana)
            .map(|(kj, fg)| KanjiReading {
                kanji: kj,
                furigana: fg.clone(),
            })
            .collect();
        Self {
            readings,
            translations: translations.collect(),
        }
    }
}

impl Default for Word {
    fn default() -> Self {
        DATABASE
            .lock()
            .expect("database is poisoned!")
            .random_word() // TODO: custom logic
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub(crate) struct KanjiReading {
    pub kanji: char,
    pub furigana: String,
}
