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
    NAME_VERSION,
    database::DATABASE,
    glyph::{self, HEIGHT_SYMBOLS, WIDTH_SYMBOLS},
    record::Word,
    ui,
};

#[derive(Default)]
pub(crate) struct MainScreen {
    word: Word,
    furigana_revealed: bool,
    translation_revealed: bool,
    selected_item: ui::Selection,
    clickables: Vec<(Rect, ui::Event)>,
}

impl MainScreen {
    /// takes the next word as by the selected kanji / furigana
    fn apply_selection(&mut self, selection: &ui::Selection) {
        match selection {
            ui::Selection::Kanji(idx) => {
                let reading = &self.word.readings[*idx];
                self.replace_word(
                    DATABASE
                        .lock()
                        .expect("DB is poisoned!")
                        .next_by_kanji(reading.kanji),
                );
            }
            ui::Selection::Furigana(idx) => {
                self.replace_word(
                    DATABASE
                        .lock()
                        .expect("DB is poisoned!")
                        .next_by_reading(&self.word.readings[*idx]),
                );
            }
        }
    }

    fn goto_next_left(&mut self) {
        self.selected_item = match self.selected_item {
            ui::Selection::Furigana(n) => ui::Selection::Furigana(n.saturating_sub(1)),
            ui::Selection::Kanji(n) => ui::Selection::Kanji(n.saturating_sub(1)),
        }
    }

    fn goto_next_right(&mut self) {
        let last_kanji_idx = self.word.readings.len() - 1;
        self.selected_item = match self.selected_item {
            ui::Selection::Furigana(n) => {
                ui::Selection::Furigana(std::cmp::min(last_kanji_idx, n + 1))
            }
            ui::Selection::Kanji(n) => ui::Selection::Kanji(std::cmp::min(last_kanji_idx, n + 1)),
        }
    }

    fn goto_next_row(&mut self) {
        if self.furigana_revealed {
            self.selected_item = match self.selected_item {
                ui::Selection::Furigana(n) => ui::Selection::Kanji(n),
                ui::Selection::Kanji(n) => ui::Selection::Furigana(n),
            }
        }
    }

    fn goto_help(&self) {
        todo!()
    }

    /// set the new word and reset the view
    fn replace_word(&mut self, new_word: Word) {
        self.word = new_word;
        self.furigana_revealed = false;
        self.translation_revealed = false;
        self.selected_item = ui::Selection::default();
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
                ui::Selection::Kanji(n) if n == idx => {
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
                .push((col, ui::Event::NextWord(ui::Selection::Kanji(idx))));
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
                    ui::Selection::Furigana(n) if n == idx => {
                        Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
                    }
                    _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
                };

                Paragraph::new(reading.furigana.as_str())
                    .block(block)
                    .centered()
                    .render(col, buf);
                self.clickables
                    .push((col, ui::Event::NextWord(ui::Selection::Furigana(idx))));
            }
        } else {
            Paragraph::new("<furigana is hidden>")
                .style(Style::new().bg(ratatui::style::Color::DarkGray))
                .centered()
                .render(area, buf);
            self.clickables.push((area, ui::Event::RevealFurigana));
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

        self.clickables.push((area, ui::Event::RevealTranslation));
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
            (KeyCode::Char('h'), _) => self.goto_help(),
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
                ui::Event::NextWord(s) => self.apply_selection(&s),
                ui::Event::RevealFurigana => self.furigana_revealed = true,
                ui::Event::RevealTranslation => self.translation_revealed = true,
            }
        }
    }
}

impl Widget for &mut MainScreen {
    /// UI layout
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.clickables = vec![];
        let intructions = Line::from(vec![
            "<1..9>".into(),
            " ".into(),
            "change screen".blue(),
            " | ".into(),
            "<f>".into(),
            " ".into(),
            "reveal furigana".blue(),
            " | ".into(),
            "<t>".into(),
            " ".into(),
            "reveal translations".blue(),
            " | ".into(),
            "<left, right, up, down arrows>".into(),
            " ".into(),
            "navigation".green(),
            " | ".into(),
            "<enter>".into(),
            " ".into(),
            "select".green(),
            " | ".into(),
            "<^C>".into(),
            " ".into(),
            "exit".red(),
            " | ".into(),
            "<h>".into(),
            " ".into(),
            "help".red(),
        ]);

        let rows = Layout::vertical([
            Constraint::Max(HEIGHT_SYMBOLS as u16 + 2), // word
            Constraint::Max(3),                         // readings
            Constraint::Fill(1),                        // translations
        ])
        .margin(1)
        .split(area)
        .to_vec();

        // outer block
        Block::bordered()
            .title(format!("{NAME_VERSION} - main"))
            .title_bottom(intructions)
            .render(area, buf);

        self.render_word(rows[0], buf);
        self.render_furigana(rows[1], buf);
        self.render_translations(rows[2], buf);
    }
}
