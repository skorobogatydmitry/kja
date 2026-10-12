// #![feature(lock_value_accessors)]

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    database::DATABASE,
    glyph::{HEIGHT_SYMBOLS, WIDTH_SYMBOLS},
    record::Word,
    ui::events::{Selection, UiEvent},
};

pub const NAME_VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

mod database;
mod glyph;
mod record;
mod ui;

#[derive(Default)]
pub struct App {
    word: Word,
    furigana_revealed: bool,
    translation_revealed: bool,
    exit: bool,
    selected_item: Selection,
    clickables: Vec<(Rect, UiEvent)>,
}

impl App {
    /// main app cycle
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            self.draw(terminal)?;
            self.handle_events()?;
            self.clickables = vec![];
        }
        Ok(())
    }

    /// frame re-drawer
    fn draw(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
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
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
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
            (KeyCode::Enter, _) => self.apply_selection(&self.selected_item.clone()),
            (KeyCode::Char('h'), _) => self.goto_help(),
            (KeyCode::Char('f'), _) => self.furigana_revealed = !self.furigana_revealed,
            (KeyCode::Char('t'), _) => self.translation_revealed = !self.translation_revealed,
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<(), anyhow::Error> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let position = Position::new(mouse_event.column, mouse_event.row);
                let event = self
                    .clickables
                    .iter()
                    .find(|(rect, _s)| rect.contains(position))
                    .map(|(_rect, s)| s)
                    .cloned();
                if let Some(e) = event {
                    match e {
                        UiEvent::NextWord(s) => self.apply_selection(&s),
                        UiEvent::RevealFurigana => self.furigana_revealed = true,
                        UiEvent::RevealTranslation => self.translation_revealed = true,
                    }
                }
            }
            _ => (), // TODO: log
        }

        Ok(())
    }

    /// takes the next word as by the selected kanji / furigana
    fn apply_selection(&mut self, selection: &Selection) {
        match selection {
            Selection::Kanji(idx) => {
                let reading = &self.word.readings[*idx];
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
                        .next_by_reading(&self.word.readings[*idx]),
                );
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
                .push((col, UiEvent::NextWord(Selection::Kanji(idx))));
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
                    .push((col, UiEvent::NextWord(Selection::Furigana(idx))));
            }
        } else {
            Paragraph::new("<furigana is hidden>")
                .style(Style::new().bg(ratatui::style::Color::DarkGray))
                .centered()
                .render(area, buf);
            self.clickables.push((area, UiEvent::RevealFurigana));
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

        self.clickables.push((area, UiEvent::RevealTranslation));
    }
}

impl Widget for &mut App {
    /// UI layout
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let intructions = Line::from(vec![
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
            "<q, ^C>".into(),
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
            .title(NAME_VERSION)
            .title_bottom(intructions)
            .render(area, buf);

        self.render_word(rows[0], buf);
        self.render_furigana(rows[1], buf);
        self.render_translations(rows[2], buf);
    }
}
