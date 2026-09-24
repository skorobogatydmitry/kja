//! word record representation
//!

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct Word {
    // TODO: words like きょう　/ 今日
    pub readings: Vec<KanjiReading>,
    pub translations: Vec<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct KanjiReading {
    pub kanji: char,
    pub furigana: String,
}
