//! word record representation
//!

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct Word {
    // TODO: words like きょう　/ 今日
    pub readings: Vec<KanjiReading>,
    pub translations: Vec<String>,
}

impl Word {
    pub(crate) fn random_word() -> Self {
        Self {
            readings: vec![
                KanjiReading {
                    kanji: '毎',
                    furigana: "まい".to_string(),
                },
                KanjiReading {
                    kanji: '日',
                    furigana: "にち".to_string(),
                },
            ],
            translations: vec!["каждый день".to_string(), "ежедневно".to_string()],
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct KanjiReading {
    pub kanji: char,
    pub furigana: String,
}
