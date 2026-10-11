// #![feature(lock_value_accessors)]

use std::{
    error::Error,
    num, process,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rakugaki::{RenderMode, render_char, render_char_with_mode};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Shadow, Widget},
};

use crate::{
    database::DATABASE,
    glyph::{HEIGHT_PX, HEIGHT_SYMBOLS, SYMBOL_HEIGHT_DOTS, WIDTH_SYMBOLS},
    record::Word,
};

pub const NAME_VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

mod database;
mod glyph;
mod record;
// mod ui;

#[derive(Debug, Default)]
pub struct App {
    word: Word,
    furigana_revealed: bool,
    translation_revealed: bool,
    exit: bool,
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
            (KeyCode::Up, _) => self.goto_next_up(),
            (KeyCode::Down, _) => self.goto_next_down(),
            (KeyCode::Char('h'), _) => self.goto_help(),
            (KeyCode::Char('f'), _) => self.furigana_revealed = !self.furigana_revealed,
            (KeyCode::Char('t'), _) => self.translation_revealed = !self.translation_revealed,

            _ => {}
        }
        Ok(())
    }

    fn goto_next_left(&self) {
        todo!()
    }

    fn goto_next_right(&self) {
        todo!()
    }

    fn goto_next_up(&self) {
        todo!()
    }

    fn goto_next_down(&self) {
        todo!()
    }

    fn goto_help(&self) {
        todo!()
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

        for (idx, row) in rows.into_iter().enumerate() {
            TopLevelRow::from((idx, self)).render(row, buf);
        }
    }
}

enum TopLevelRow {
    Word(Vec<char>),
    Furigana(Option<Vec<String>>),
    Translations(Option<Vec<String>>),
}

impl TopLevelRow {
    fn render_word(kanjis: Vec<char>, area: Rect, buf: &mut Buffer) {
        let num_kanjis = kanjis.len();
        let outer_layout =
            Layout::horizontal((0..num_kanjis).map(|_| Constraint::Min(WIDTH_SYMBOLS as u16 + 2)))
                .spacing(3)
                .flex(ratatui::layout::Flex::Center)
                .split(area)
                .to_vec();

        for (kj, col) in kanjis.iter().zip(outer_layout) {
            let kanji_block = Block::bordered().border_type(ratatui::widgets::BorderType::Rounded);

            let rendered_kanji_lines = glyph::render(*kj);
            // let text = Text
            Paragraph::new(rendered_kanji_lines.join("\n"))
                .block(kanji_block)
                .centered()
                .render(col, buf);
        }
    }

    fn render_furigana(furiganas: Option<Vec<String>>, area: Rect, buf: &mut Buffer) {
        match furiganas {
            None => {
                Paragraph::new("<furigana is hidden>")
                    // .block(block)
                    .centered()
                    .render(area, buf);
            }
            Some(furiganas) => {
                let num_kanjis = furiganas.len();
                let outer_layout = Layout::horizontal(
                    (0..num_kanjis).map(|_| Constraint::Min(WIDTH_SYMBOLS as u16 + 2)),
                )
                .spacing(3)
                .flex(ratatui::layout::Flex::Center)
                .split(area)
                .to_vec();

                for (furigana, col) in furiganas.into_iter().zip(outer_layout) {
                    let block =
                        Block::bordered().border_type(ratatui::widgets::BorderType::Rounded);

                    // let rendered_kanji_lines = glyph::render(*kj);
                    // let text = Text
                    Paragraph::new(furigana.clone())
                        .block(block)
                        .centered()
                        .render(col, buf);
                }
            }
        }
    }
}

impl Widget for TopLevelRow {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            Self::Word(w) => Self::render_word(w, area, buf),
            Self::Furigana(furiganas) => Self::render_furigana(furiganas, area, buf),
            _ => Paragraph::new("Not there yet")
                .centered()
                .block(Block::bordered())
                .render(area, buf),
        }
    }
}

impl From<(usize, &App)> for TopLevelRow {
    fn from(value: (usize, &App)) -> Self {
        let (row_idx, app) = value;
        match row_idx {
            0 => Self::Word(app.word.readings.iter().map(|kr| kr.kanji).collect()),
            1 => Self::Furigana(
                app.furigana_revealed.then_some(
                    app.word
                        .readings
                        .iter()
                        .map(|kr| kr.furigana.clone())
                        .collect(),
                ),
            ),
            2 => Self::Translations(
                app.translation_revealed
                    .then_some(app.word.translations.iter().cloned().collect()),
            ),
            _ => panic!("more rows on the top level than expected"),
        }
    }
}
//         egui_extras::install_image_loaders(&cc.egui_ctx);

//         let mut fonts = FontDefinitions::default();
//         let jp_font_name = String::from("KleeOne Regular");
//         fonts.font_data.insert(
//             jp_font_name.clone(),
//             std::sync::Arc::new(FontData::from_static(JP_FONT)),
//         );
//         fonts
//             .families
//             .get_mut(&egui::FontFamily::Proportional)
//             .unwrap()
//             .push(jp_font_name);
//         cc.egui_ctx.set_fonts(fonts);

//         Ok(Box::new(Self {
//             // TODO: smart / on-demand algo on choosing
//             word: Arc::new(Mutex::new(
//                 DATABASE
//                     .lock()
//                     .expect("database is poisoned!")
//                     .random_word(),
//             )),
//             furigana_revealed: Mutex::new(false),
//             translation_revealed: Mutex::new(false),
//         }))
//     }

//     fn top_bar(&mut self, strip: &mut Strip) {
//         strip.strip(|builder| {
//             builder
//                 .clip(true)
//                 .size(Size::remainder()) // TODO: dynamic calc
//                 .horizontal(|mut strip| {
//                     strip.cell(|ui| {
//                         ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
//                             let info_popup_button = ui
//                                 .button(egui::include_image!("../res/question.png"))
//                                 .on_hover_text("info");
//                             Popup::menu(&info_popup_button)
//                                 .gap(5.0)
//                                 .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
//                                 .show(|ui| {
//                                     ScrollArea::both()
//                                         .show(ui, |ui| ui.label(include_str!("../README.md")));
//                                 });
//                             if ui
//                                 .button(egui::include_image!("../res/cog.png"))
//                                 .on_hover_text("settings")
//                                 .clicked()
//                             {
//                                 self.settings_menu(ui);
//                             }

//                             let add_kanji_button = ui
//                                 .button(egui::include_image!("../res/add.png"))
//                                 .on_hover_text("add a new kanji");
//                             ui::add_kanji::popup(&add_kanji_button);
//                         });
//                     });
//                 });
//         });
//     }

//     fn settings_menu(&self, ui: &mut egui::Ui) {
//         todo!()
//     }
// }

// impl eframe::App for App {
//     fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
//         let current_word = self.word.get_cloned().expect("UI state is poisoned");
//         let mut is_furigana_revealed = self
//             .furigana_revealed
//             .get_cloned()
//             .expect("UI state it posoned");

//         let mut is_translation_revealed = self
//             .translation_revealed
//             .get_cloned()
//             .expect("UI state it posoned");

//         StripBuilder::new(ui)
//             .size(Size::relative(0.2))
//             .sizes(Size::remainder(), 3)
//             .vertical(|mut strip| {
//                 self.top_bar(&mut strip);
//                 strip.strip(|builder| {
//                     builder
//                         .sizes(Size::remainder(), current_word.readings.len())
//                         .horizontal(|mut strip| {
//                             for symbol in current_word.readings.iter() {
//                                 strip.cell(|ui| {
//                                     ui.centered_and_justified(|ui| {
//                                         if ui
//                                             .button(
//                                                 egui::RichText::new(symbol.kanji)
//                                                     .size(32.0)
//                                                     .heading()
//                                                     .strong(),
//                                             )
//                                             .highlight()
//                                             .clicked()
//                                         {
//                                             self.replace_word(
//                                                 DATABASE
//                                                     .lock()
//                                                     .expect("DB is poisoned!")
//                                                     .next_by_kanji(symbol.kanji),
//                                             );
//                                         }
//                                     });
//                                 });
//                             }
//                         });
//                 });
//                 if is_furigana_revealed {
//                     strip.strip(|builder| {
//                         builder
//                             .sizes(Size::remainder(), current_word.readings.len())
//                             .horizontal(|mut strip| {
//                                 for reading in current_word.readings.iter() {
//                                     strip.cell(|ui| {
//                                         ui.centered_and_justified(|ui| {
//                                             if ui
//                                                 .button(
//                                                     egui::RichText::new(reading.furigana.as_str())
//                                                         .size(22.0),
//                                                 )
//                                                 .clicked()
//                                             {
//                                                 self.replace_word(
//                                                     DATABASE
//                                                         .lock()
//                                                         .expect("DB is poisoned!")
//                                                         .next_by_reading(reading),
//                                                 );
//                                             }
//                                         });
//                                     });
//                                 }
//                             });
//                     });
//                 } else {
//                     strip.cell(|ui| {
//                         ui.centered_and_justified(|ui| {
//                             ui.scope(|ui| {
//                                 ui.set_max_width(120.0);
//                                 if ui.button("reveal reading").clicked() {
//                                     self.furigana_revealed
//                                         .replace(true)
//                                         .expect("UI state is poisoned");
//                                     is_furigana_revealed = true;
//                                 }
//                             });
//                         });
//                     });
//                 }

//                 if is_translation_revealed {
//                     strip.strip(|builder| {
//                         builder
//                             .sizes(Size::remainder(), current_word.translations.len())
//                             .horizontal(|mut strip| {
//                                 for translation in current_word.translations.iter() {
//                                     strip.cell(|ui| {
//                                         ui.centered_and_justified(|ui| {
//                                             ui.label(
//                                                 egui::RichText::new(translation.as_str())
//                                                     .size(22.0),
//                                             )
//                                         });
//                                     });
//                                 }
//                             });
//                     });
//                 } else {
//                     strip.cell(|ui| {
//                         ui.centered_and_justified(|ui| {
//                             ui.scope(|ui| {
//                                 ui.set_max_width(120.0);
//                                 if ui.button("reveal translation").clicked() {
//                                     self.translation_revealed
//                                         .replace(true)
//                                         .expect("UI state is poisoned");
//                                     is_translation_revealed = true;
//                                 }
//                             });
//                         });
//                     });
//                 }
//             });
//     }
// }

// impl App {
//     fn replace_word(&mut self, new_word: Word) {
//         self.furigana_revealed
//             .replace(false)
//             .expect("UI state is poisoned");
//         self.translation_revealed
//             .replace(false)
//             .expect("UI state is poisoned");
//         self.word.replace(new_word).expect("UI state is poisoned");
//     }
// }
