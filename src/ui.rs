/// Mouse event types to handle
#[derive(Clone)]
pub(crate) enum Event {
    NextWord(Selection),
    RevealFurigana,
    RevealTranslation,
}

//
#[derive(Debug, Clone)]
pub(crate) enum Selection {
    Kanji(usize),
    Furigana(usize),
}

impl Default for Selection {
    fn default() -> Self {
        Self::Kanji(0)
    }
}
