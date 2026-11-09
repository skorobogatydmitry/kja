//! word record representation
//!

use std::{collections::HashSet, sync::LazyLock, vec::Drain};

use icu_properties::{CodePointMapData, CodePointMapDataBorrowed, props::Script};

use crate::database::Database;
static CODE_POINT_MAP_DATA: LazyLock<CodePointMapDataBorrowed<Script>> =
    LazyLock::new(CodePointMapData::<Script>::new);

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct Word {
    // TODO: words like きょう　/ 今日
    pub readings: Vec<KanjiReading>,
    pub translations: Vec<String>,
}

/// make a word from list of kanjis and readings
impl<'a> TryFrom<(Drain<'a, char>, Drain<'a, String>, Drain<'a, String>)> for Word {
    type Error = ErrorWordFromTuple;
    fn try_from(value: (Drain<char>, Drain<String>, Drain<String>)) -> Result<Self, Self::Error> {
        let (kanjis, furiganas, translations) = value;
        for k in kanjis.as_ref() {
            if !is_kanji(k) {
                return Err(ErrorWordFromTuple::NotKanji(*k));
            }
        }
        if kanjis.len() < 1 {
            return Err(ErrorWordFromTuple::TooFewKanjis);
        }
        if kanjis.len() != furiganas.len() || kanjis.len() < 1 {
            return Err(ErrorWordFromTuple::KanjiCountMismatch);
        }

        if let Some(incorrect_furigana) = furiganas
            .as_ref()
            .iter()
            .find(|f| f.is_empty() || !is_consistent_kana(f))
        {
            return Err(ErrorWordFromTuple::InconsistentFurigana(
                incorrect_furigana.to_string(),
            ));
        }

        let readings = kanjis
            .zip(furiganas)
            .map(|(kj, fg)| KanjiReading {
                kanji: kj,
                furigana: fg.clone(),
            })
            .collect();
        Ok(Self {
            readings,
            translations: translations.collect(),
        })
    }
}

impl Default for Word {
    fn default() -> Self {
        Database::random_word() // TODO: custom logic
    }
}

pub enum ErrorWordFromTuple {
    NotKanji(char),
    TooFewKanjis,
    KanjiCountMismatch,
    InconsistentFurigana(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub(crate) struct KanjiReading {
    pub kanji: char,
    pub furigana: String,
}

/// check if the character is kanji
pub(crate) fn is_kanji(ch: &char) -> bool {
    CODE_POINT_MAP_DATA.get(*ch) == Script::Han
}

/// check if the character is kana
pub(crate) fn is_kana(ch: char) -> bool {
    CODE_POINT_MAP_DATA.get(ch) == Script::Hiragana
        || CODE_POINT_MAP_DATA.get(ch) == Script::Katakana
}

/// check that the word contains only 1 type of kana
pub(crate) fn is_consistent_kana(candidate: &str) -> bool {
    let char_types = candidate
        .chars()
        .map(|ch| CODE_POINT_MAP_DATA.get(ch))
        .collect::<HashSet<_>>();
    if char_types.len() != 1 {
        return false;
    } else {
        let uniq_type = char_types.into_iter().next().unwrap();
        uniq_type == Script::Hiragana || uniq_type == Script::Katakana
    }
}
