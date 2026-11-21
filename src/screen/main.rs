/// the main screen with word reading guessing
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    database::Database,
    glyph::{self, HEIGHT_SYMBOLS, WIDTH_SYMBOLS},
    word::Word,
};

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

#[derive(Default)]
pub(crate) struct MainScreen {
    word: Word,
    furigana_revealed: bool,
    translation_revealed: bool,
    selected_item: Selection,
    clickables: Vec<(Rect, Event)>,
}

impl MainScreen {
    /// takes the next word as by the selected kanji / furigana
    fn apply_selection(&mut self, selection: &Selection) {
        match selection {
            Selection::Kanji(idx) => {
                let reading = &self.word.readings[*idx];
                self.replace_word(Database::next_by_kanji(reading.kanji));
            }
            Selection::Furigana(idx) => {
                self.replace_word(Database::next_by_reading(&self.word.readings[*idx]));
            }
        }
    }

    fn goto_next_left(&mut self) {
        self.selected_item = match self.selected_item {
            Selection::Furigana(n) => Selection::Furigana(n.saturating_sub(1)),
            Selection::Kanji(n) => Selection::Kanji(n.saturating_sub(1)),
        }
    }

    fn goto_next_right(&mut self) {
        let last_kanji_idx = self.word.readings.len() - 1;
        self.selected_item = match self.selected_item {
            Selection::Furigana(n) => Selection::Furigana(std::cmp::min(last_kanji_idx, n + 1)),
            Selection::Kanji(n) => Selection::Kanji(std::cmp::min(last_kanji_idx, n + 1)),
        }
    }

    fn goto_next_row(&mut self) {
        if self.furigana_revealed {
            self.selected_item = match self.selected_item {
                Selection::Furigana(n) => Selection::Kanji(n),
                Selection::Kanji(n) => Selection::Furigana(n),
            }
        }
    }

    /// set the new word and reset the view
    fn replace_word(&mut self, new_word: Word) {
        self.word = new_word;
        self.furigana_revealed = false;
        self.translation_revealed = false;
        self.selected_item = Selection::default();
    }

    fn render_word(&mut self, area: Rect, buf: &mut Buffer) {
        let num_kanjis = self.word.readings.len();
        let outer_layout =
            Layout::horizontal((0..num_kanjis).map(|_| Constraint::Min(WIDTH_SYMBOLS as u16 + 2)))
                .spacing(3)
                .flex(ratatui::layout::Flex::Center)
                .split(area)
                .to_vec();

        for (idx, (reading, col)) in self.word.readings.iter().zip(outer_layout).enumerate() {
            let kanji_block = match self.selected_item {
                Selection::Kanji(n) if n == idx => {
                    Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
                }
                _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
            };

            let rendered_kanji_lines = glyph::render(reading.kanji);
            Paragraph::new(rendered_kanji_lines.join("\n"))
                .block(kanji_block)
                .centered()
                .render(col, buf);
            self.clickables
                .push((col, Event::NextWord(Selection::Kanji(idx))));
        }
    }

    fn render_furigana(&mut self, area: Rect, buf: &mut Buffer) {
        if self.furigana_revealed {
            let num_kanjis = self.word.readings.len();
            let outer_layout = Layout::horizontal(
                (0..num_kanjis).map(|_| Constraint::Min(WIDTH_SYMBOLS as u16 + 2)),
            )
            .spacing(3)
            .flex(ratatui::layout::Flex::Center)
            .split(area)
            .to_vec();

            for (idx, (reading, col)) in self.word.readings.iter().zip(outer_layout).enumerate() {
                let block = match self.selected_item {
                    Selection::Furigana(n) if n == idx => {
                        Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
                    }
                    _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
                };

                Paragraph::new(reading.furigana.as_str())
                    .block(block)
                    .centered()
                    .render(col, buf);
                self.clickables
                    .push((col, Event::NextWord(Selection::Furigana(idx))));
            }
        } else {
            Paragraph::new("<furigana is hidden>")
                .style(Style::new().bg(ratatui::style::Color::DarkGray))
                .centered()
                .render(area, buf);
            self.clickables.push((area, Event::RevealFurigana));
        }
    }

    fn render_translations(&mut self, area: Rect, buf: &mut Buffer) {
        let paragraph = self
            .translation_revealed
            .then_some(self.word.translations.clone())
            .map(|ts| Paragraph::new(ts.join(", ")))
            .unwrap_or(
                Paragraph::new("<translations are hidden>")
                    .style(Style::new().bg(ratatui::style::Color::DarkGray)),
            );

        paragraph
            .block(Block::bordered())
            .centered()
            .render(area, buf);

        self.clickables.push((area, Event::RevealTranslation));
    }

    pub(crate) fn handle_key_event(
        &mut self,
        code_and_modifier: (crossterm::event::KeyCode, crossterm::event::KeyModifiers),
    ) {
        match code_and_modifier {
            (KeyCode::Left, _) => self.goto_next_left(),
            (KeyCode::Right, _) => self.goto_next_right(),
            (KeyCode::Up, _) => self.goto_next_row(),
            (KeyCode::Down, _) => self.goto_next_row(),
            (KeyCode::Enter, _) => self.apply_selection(&self.selected_item.clone()),
            (KeyCode::Char('f'), _) => self.furigana_revealed = !self.furigana_revealed,
            (KeyCode::Char('t'), _) => self.translation_revealed = !self.translation_revealed,
            _ => (), // TODO: log
        }
    }

    pub(crate) fn handle_mouse_event(&mut self, position: Position) {
        let event = self
            .clickables
            .iter()
            .find(|(rect, _s)| rect.contains(position))
            .map(|(_rect, s)| s)
            .cloned();
        if let Some(e) = event {
            match e {
                Event::NextWord(s) => self.apply_selection(&s),
                Event::RevealFurigana => self.furigana_revealed = true,
                Event::RevealTranslation => self.translation_revealed = true,
            }
        }
    }
}

impl crate::screen::Screen for MainScreen {
    fn name(&self) -> &'static str {
        "Main"
    }
    fn custom_footer(&self) -> Line<'static> {
        Line::from(vec![
            "<f>".into(),
            " ".into(),
            "show furigana".blue(),
            " | ".into(),
            "<t>".into(),
            " ".into(),
            "show translations".blue(),
            " | ".into(),
            "<enter>".into(),
            " ".into(),
            "next word by".green(),
        ])
    }

    fn render_body(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.clickables = vec![];

        let rows = Layout::vertical([
            Constraint::Max(HEIGHT_SYMBOLS as u16 + 2), // word
            Constraint::Max(3),                         // readings
            Constraint::Max(3),                         // translations
        ])
        .margin(1)
        .split(area)
        .to_vec();

        self.render_word(rows[0], buf);
        self.render_furigana(rows[1], buf);
        self.render_translations(rows[2], buf);
    }
}
