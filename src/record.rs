//! word record representation
//!

use crate::DATABASE;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct Word {
    // TODO: words like きょう　/ 今日
    pub readings: Vec<KanjiReading>,
    pub translations: Vec<String>,
}

impl Default for Word {
    fn default() -> Self {
        DATABASE
            .lock()
            .expect("database is poisoned!")
            .random_word()
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub(crate) struct KanjiReading {
    pub kanji: char,
    pub furigana: String,
}
