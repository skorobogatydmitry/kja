/// the screen to add a new kanji to database
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{NAME_VERSION, database::DATABASE, record::Word, ui};

#[derive(Clone)]
enum Event {
    Focus(Selection),
}

/// Current selected item on the screen
#[derive(Debug, Clone)]
enum Selection {
    Kanji(usize),
    Furigana(usize),
    Translation(usize),
    AddWord,
}

impl Default for Selection {
    fn default() -> Self {
        Self::Kanji(0)
    }
}

#[derive(Default)]
pub(crate) struct Screen {
    kanjis: Vec<char>,
    furiganas: Vec<String>, // TODO: hint from DB
    translations: Vec<String>,
    selected_item: Selection,
    clickables: Vec<(Rect, Event)>,
}

impl Screen {
    /// takes the next word as by the selected kanji / furigana
    fn apply_selection(&mut self, selection: &ui::Selection) {
        // match selection {
        //     ui::Selection::Kanji(idx) => {
        //         let reading = &self.word.readings[*idx];
        //         self.replace_word(
        //             DATABASE
        //                 .lock()
        //                 .expect("DB is poisoned!")
        //                 .next_by_kanji(reading.kanji),
        //         );
        //     }
        //     ui::Selection::Furigana(idx) => {
        //         self.replace_word(
        //             DATABASE
        //                 .lock()
        //                 .expect("DB is poisoned!")
        //                 .next_by_reading(&self.word.readings[*idx]),
        //         );
        //     }
        // }
    }

    fn goto_next_left(&mut self) {
        self.selected_item = match self.selected_item {
            Selection::Kanji(n) => Selection::Kanji(n.saturating_sub(1)),
            Selection::Furigana(n) => Selection::Furigana(n.saturating_sub(1)),
            Selection::Translation(n) => Selection::Translation(n.saturating_sub(1)),
            Selection::AddWord => Selection::Translation(self.translations.len()),
        }
    }

    fn goto_next_right(&mut self) {
        self.selected_item = match self.selected_item {
            Selection::Kanji(n) => Selection::Kanji(std::cmp::min(n + 1, self.kanjis.len())),
            Selection::Furigana(n) => Selection::Furigana(std::cmp::min(n + 1, self.kanjis.len())),
            Selection::Translation(n) => {
                if n == self.translations.len() {
                    Selection::AddWord
                } else {
                    Selection::Translation(std::cmp::min(n + 1, self.translations.len()))
                }
            }
            Selection::AddWord => Selection::AddWord,
        }
    }

    fn goto_next_row(&mut self) {
        self.selected_item = match self.selected_item {
            Selection::Kanji(n) => Selection::Furigana(n),
            Selection::Furigana(n) => {
                if n > self.translations.len() {
                    Selection::AddWord
                } else {
                    Selection::Translation(std::cmp::min(self.translations.len(), n))
                }
            }
            Selection::Translation(_n) => Selection::Kanji(0),
            Selection::AddWord => Selection::Kanji(0),
        }
    }

    fn goto_prev_row(&mut self) {
        self.selected_item = match self.selected_item {
            Selection::Kanji(n) => {
                Selection::Translation(std::cmp::min(self.translations.len(), n))
            }
            Selection::Furigana(n) => Selection::Kanji(n),
            Selection::Translation(n) => Selection::Furigana(std::cmp::min(self.kanjis.len(), n)),
            Selection::AddWord => Selection::Furigana(std::cmp::min(
                self.kanjis.len(),
                self.translations.len() + 1,
            )),
        }
    }

    fn add_word(&mut self) {
        // TODO: validate
        let word = Word::from_input(
            self.kanjis.drain(..),
            self.furiganas.drain(..),
            self.translations.drain(..),
        );
        match DATABASE
            .lock()
            .expect("database is poisoned")
            .add_word(word)
        {
            Ok(_) => (),   // TODO: log
            Err(_e) => (), // TODO: log
        }
    }

    fn next_field(&mut self) {
        self.selected_item = match self.selected_item {
            Selection::Kanji(n) => {
                if n < self.kanjis.len() {
                    Selection::Kanji(n + 1)
                } else {
                    Selection::Furigana(0)
                }
            }
            Selection::Furigana(n) => {
                if n < self.kanjis.len() {
                    Selection::Furigana(n + 1)
                } else {
                    Selection::Translation(0)
                }
            }
            Selection::Translation(n) => {
                if n == self.translations.len() {
                    Selection::AddWord
                } else if n < self.translations.len() {
                    Selection::Translation(n + 1)
                } else {
                    Selection::Kanji(0)
                }
            }
            Selection::AddWord => Selection::Kanji(0),
        }
    }

    fn render_kanjis(&mut self, area: Rect, buf: &mut Buffer) {
        let num_kanjis = self.kanjis.len();
        let mut outer_layout = Layout::horizontal((0..=num_kanjis).map(|_| Constraint::Fill(1)))
            .spacing(3)
            .flex(ratatui::layout::Flex::Center)
            .split(area)
            .to_vec();
        let last_col = outer_layout.pop().unwrap(); // there's at least 1 column placeholder

        for (idx, (kanji, col)) in self.kanjis.iter().zip(outer_layout).enumerate() {
            let kanji_block = match self.selected_item {
                Selection::Kanji(n) if n == idx => {
                    Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
                }
                _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
            };

            Paragraph::new(kanji.to_string())
                .block(kanji_block)
                .centered()
                .render(col, buf);
            self.clickables
                .push((col, Event::Focus(Selection::Kanji(idx))));
        }

        // the last empty block for new kanji
        let kanji_block = match self.selected_item {
            Selection::Kanji(n) if n == num_kanjis => {
                Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
            }
            _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
        };

        Paragraph::new("__")
            .block(kanji_block)
            .centered()
            .render(last_col, buf);
        self.clickables
            .push((last_col, Event::Focus(Selection::Kanji(num_kanjis))));
    }

    fn render_furigana(&mut self, area: Rect, buf: &mut Buffer) {
        let num_furiganas = self.furiganas.len();
        let mut outer_layout = Layout::horizontal((0..=num_furiganas).map(|_| Constraint::Fill(1)))
            .spacing(3)
            .flex(ratatui::layout::Flex::Center)
            .split(area)
            .to_vec();

        let last_col = outer_layout.pop().unwrap(); // there's at least 1 col

        for (idx, (furigana, col)) in self.furiganas.iter().zip(outer_layout).enumerate() {
            let block = match self.selected_item {
                Selection::Furigana(n) if n == idx => {
                    Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
                }
                _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
            };

            Paragraph::new(furigana.as_str())
                .block(block)
                .centered()
                .render(col, buf);
            self.clickables
                .push((col, Event::Focus(Selection::Furigana(idx))));
        }

        let block = match self.selected_item {
            Selection::Furigana(n) if n == num_furiganas => {
                Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
            }
            _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
        };

        Paragraph::new("__")
            .block(block)
            .centered()
            .render(last_col, buf);
        self.clickables
            .push((last_col, Event::Focus(Selection::Furigana(num_furiganas))));
    }

    fn render_translations(&mut self, area: Rect, buf: &mut Buffer) {
        let num_translations = self.translations.len();
        let mut constraints = (0..=num_translations + 1)
            .map(|_| Constraint::Fill(1))
            .collect::<Vec<Constraint>>();
        constraints.push(Constraint::Percentage(10));
        let mut outer_layout = Layout::horizontal(constraints)
            .spacing(3)
            .flex(ratatui::layout::Flex::Center)
            .split(area)
            .to_vec();

        let add_button_col = outer_layout.pop().unwrap(); // there's at least 2 cols
        let add_translation_col = outer_layout.pop().unwrap(); // there's at least 2 cols

        for (idx, (translation, col)) in self.translations.iter().zip(outer_layout).enumerate() {
            let block = match self.selected_item {
                Selection::Translation(n) if n == idx => {
                    Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
                }
                _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
            };

            Paragraph::new(translation.as_str())
                .block(block)
                .centered()
                .render(col, buf);
            self.clickables
                .push((col, Event::Focus(Selection::Translation(idx))));
        }

        let add_translation_block = match self.selected_item {
            Selection::Translation(n) if n == num_translations => {
                Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
            }
            _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
        };

        Paragraph::new("<+1 translation>")
            .block(add_translation_block)
            .centered()
            .render(add_translation_col, buf);
        self.clickables
            .push((add_translation_col, Event::Focus(Selection::AddWord)));

        let button_block = match self.selected_item {
            Selection::AddWord => {
                Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
            }
            _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
        };

        Paragraph::new("入れる")
            .block(button_block)
            .centered()
            .render(add_button_col, buf);
        self.clickables
            .push((add_button_col, Event::Focus(Selection::AddWord)));
    }

    pub(crate) fn handle_key_event(
        &mut self,
        code_and_modifier: (crossterm::event::KeyCode, crossterm::event::KeyModifiers),
    ) {
        match code_and_modifier {
            (KeyCode::Up, _) => self.goto_prev_row(),
            (KeyCode::Down, _) => self.goto_next_row(),
            (KeyCode::Left, _) => self.goto_next_left(),
            (KeyCode::Right, _) => self.goto_next_right(),
            (KeyCode::Enter, _) => self.add_word(),
            (KeyCode::Tab, _) => self.next_field(),
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
                Event::Focus(Selection::AddWord) => todo!(),
                _ => todo!(),
            }
        }
    }
}

impl Widget for &mut Screen {
    /// UI layout
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.clickables = vec![];
        let intructions = Line::from(vec![
            // TODO: inject common hotkeys
            "<left, right, up, down arrows>".into(),
            " ".into(),
            "navigation".green(),
            " | ".into(),
            "<enter>".into(),
            " ".into(),
            "add kanji".green(),
            " | ".into(),
            "<tab>".into(),
            " ".into(),
            "next field".red(),
        ]);

        let rows = Layout::vertical([
            Constraint::Fill(1), // word
            Constraint::Fill(1), // readings
            Constraint::Fill(1), // translations
        ])
        .margin(1)
        .split(area)
        .to_vec();

        // outer block
        Block::bordered()
            .title(NAME_VERSION)
            .title_bottom(intructions)
            .render(area, buf);

        self.render_kanjis(rows[0], buf);
        self.render_furigana(rows[1], buf);
        self.render_translations(rows[2], buf);
    }
}
