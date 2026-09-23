//! word record representation
//!

#[derive(Debug, Clone)]
pub(crate) struct Word {
    // TODO: words like きょう　/ 今日
    pub readings: Vec<KanjiReading>,
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
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct KanjiReading {
    pub kanji: char,
    pub furigana: String,
}
