// #![feature(lock_value_accessors)]

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    database::DATABASE,
    glyph::{HEIGHT_SYMBOLS, WIDTH_SYMBOLS},
    record::Word,
};

pub const NAME_VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

mod database;
mod glyph;
mod record;
// mod ui;

#[derive(Debug)]
enum Selection {
    Kanji(usize),
    Furigana(usize),
}

impl Default for Selection {
    fn default() -> Self {
        Self::Kanji(0)
    }
}

#[derive(Debug, Default)]
pub struct App {
    word: Word,
    furigana_revealed: bool,
    translation_revealed: bool,
    exit: bool,
    selected_item: Selection,
}

impl App {
    /// main app cycle
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            self.draw(terminal)?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// frame re-drawer
    fn draw(&self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(())
    }

    /// user event handler
    fn handle_events(&mut self) -> anyhow::Result<()> {
        // TODO: non-blocking poll instead
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => Ok(()), // TODO: log
        }
    }

    /// event handlers router
    fn handle_key_event(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        match (event.code, event.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.exit = true
            }
            (KeyCode::Left, _) => self.goto_next_left(),
            (KeyCode::Right, _) => self.goto_next_right(),
            (KeyCode::Up, _) => self.goto_next_row(),
            (KeyCode::Down, _) => self.goto_next_row(),
            (KeyCode::Enter, _) => {
                match self.selected_item {
                    Selection::Kanji(idx) => {
                        let reading = &self.word.readings[idx];
                        self.replace_word(
                            DATABASE
                                .lock()
                                .expect("DB is poisoned!")
                                .next_by_kanji(reading.kanji),
                        );
                    }
                    Selection::Furigana(idx) => {
                        self.replace_word(
                            DATABASE
                                .lock()
                                .expect("DB is poisoned!")
                                .next_by_reading(&self.word.readings[idx]),
                        );
                    }
                };
            }
            (KeyCode::Char('h'), _) => self.goto_help(),
            (KeyCode::Char('f'), _) => self.furigana_revealed = !self.furigana_revealed,
            (KeyCode::Char('t'), _) => self.translation_revealed = !self.translation_revealed,
            (KeyCode::Char(ch), _) => {
                if ch.is_digit(10) {
                    let num_pressed = ch.to_digit(10).unwrap();
                    match self.word.readings.get(num_pressed as usize - 1) {
                        Some(reading) => {
                            self.replace_word(
                                DATABASE
                                    .lock()
                                    .expect("DB is poisoned!")
                                    .next_by_kanji(reading.kanji),
                            );
                        }
                        None => (), // TODO: show warning
                    }
                }
            }

            _ => {}
        }
        Ok(())
    }

    fn goto_next_left(&mut self) {
        self.selected_item = match self.selected_item {
            Selection::Furigana(n) => Selection::Furigana(n.saturating_sub(1)),
            Selection::Kanji(n) => Selection::Kanji(n.saturating_sub(1)),
        }
    }

    fn goto_next_right(&mut self) {
        let kanjis_in_word = self.word.readings.len() - 1;
        self.selected_item = match self.selected_item {
            Selection::Furigana(n) => Selection::Furigana(std::cmp::min(kanjis_in_word, n + 1)),
            Selection::Kanji(n) => Selection::Kanji(std::cmp::min(kanjis_in_word, n + 1)),
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

    fn goto_help(&self) {
        todo!()
    }

    /// set the new word and reset the view
    fn replace_word(&mut self, new_word: Word) {
        self.word = new_word;
        self.furigana_revealed = false;
        self.translation_revealed = false;
        self.selected_item = Selection::default();
    }

    fn render_word(&self, area: Rect, buf: &mut Buffer) {
        let num_kanjis = self.word.readings.len();
        let outer_layout =
            Layout::horizontal((0..num_kanjis).map(|_| Constraint::Min(WIDTH_SYMBOLS as u16 + 2)))
                .spacing(3)
                .flex(ratatui::layout::Flex::Center)
                .split(area)
                .to_vec();

        for (idx, (kj, col)) in self.word.readings.iter().zip(outer_layout).enumerate() {
            let kanji_block = match self.selected_item {
                Selection::Kanji(n) if n == idx => {
                    Block::bordered().border_type(ratatui::widgets::BorderType::QuadrantOutside)
                }
                _ => Block::bordered().border_type(ratatui::widgets::BorderType::Rounded),
            };

            let rendered_kanji_lines = glyph::render(kj.kanji);
            Paragraph::new(rendered_kanji_lines.join("\n"))
                .block(kanji_block)
                .centered()
                .render(col, buf);
        }
    }

    fn render_furigana(&self, area: Rect, buf: &mut Buffer) {
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
            }
        } else {
            Paragraph::new("<furigana is hidden>")
                .style(Style::new().bg(ratatui::style::Color::DarkGray))
                .centered()
                .render(area, buf);
        }
    }

    fn render_translations(&self, area: Rect, buf: &mut Buffer) {
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
    }
}

impl Widget for &App {
    /// UI layout
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let intructions = Line::from(vec![
            "<f>".into(),
            " ".into(),
            "reveal furigana".blue().into(),
            " | ".into(),
            "<t>".into(),
            " ".into(),
            "reveal translations".blue().into(),
            " | ".into(),
            "<left, right, up, down arrows>".into(),
            " ".into(),
            "navigation".green().into(),
            " | ".into(),
            "<enter>".into(),
            " ".into(),
            "select".green().into(),
            " | ".into(),
            "<q, ^C>".into(),
            " ".into(),
            "exit".red().into(),
            " | ".into(),
            "<h>".into(),
            " ".into(),
            "help".red().into(),
        ]);
        // whole screen
        let outer_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(0)])
            .split(area)[0];

        let rows = Layout::vertical([
            Constraint::Max(HEIGHT_SYMBOLS as u16 + 2), // word
            Constraint::Max(3),                         // readings
            Constraint::Fill(1),                        // translations
        ])
        .margin(1)
        .split(outer_layout)
        .to_vec();

        // outer block
        Block::bordered()
            .title(NAME_VERSION)
            .title_bottom(intructions)
            .render(area, buf);

        self.render_word(rows[0], buf);
        self.render_furigana(rows[1], buf);
        self.render_translations(rows[2], buf);
    }
}
