#![feature(lock_value_accessors)]

use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use eframe::CreationContext;
use egui::{FontData, FontDefinitions};
use egui_extras::{Size, StripBuilder};

use crate::{database::DATABASE, record::Word};

pub const NAME_VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
static JP_FONT: &[u8] = include_bytes!("../res/KleeOne-Regular.ttf");

mod database;
mod record;

pub struct App {
    word: Arc<Mutex<Word>>,
    furigana_revealed: Mutex<bool>,
    translation_revealed: Mutex<bool>,
}

impl App {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(cc: &CreationContext) -> Result<Box<dyn eframe::App>, Box<dyn Error + Send + Sync>> {
        // cc.egui_ctx.set_zoom_factor(Settings::get().zoom_factor);

        let mut fonts = FontDefinitions::default();
        let jp_font_name = String::from("KleeOne Regular");
        fonts.font_data.insert(
            jp_font_name.clone(),
            std::sync::Arc::new(FontData::from_static(JP_FONT)),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push(jp_font_name);
        cc.egui_ctx.set_fonts(fonts);

        Ok(Box::new(Self {
            // TODO: smart / on-demand algo on choosing
            word: Arc::new(Mutex::new(
                DATABASE
                    .lock()
                    .expect("database is poisoned!")
                    .random_word(),
            )),
            furigana_revealed: Mutex::new(false),
            translation_revealed: Mutex::new(false),
        }))
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let current_word = self.word.get_cloned().expect("UI state is poisoned");
        let mut is_furigana_revealed = self
            .furigana_revealed
            .get_cloned()
            .expect("UI state it posoned");

        let mut is_translation_revealed = self
            .translation_revealed
            .get_cloned()
            .expect("UI state it posoned");

        StripBuilder::new(ui)
            .size(Size::remainder()) // the word
            .size(Size::exact(32.0)) // furigana revealing button
            .size(Size::remainder()) // furigana
            .size(Size::exact(32.0)) // translations revealing button
            .size(Size::remainder()) // translations
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .sizes(Size::remainder(), current_word.readings.len())
                        .horizontal(|mut strip| {
                            for symbol in current_word.readings.iter() {
                                strip.cell(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        if ui
                                            .button(egui::RichText::new(symbol.kanji).size(32.0))
                                            .clicked()
                                        {
                                            self.replace_word(
                                                DATABASE
                                                    .lock()
                                                    .expect("DB is poisoned!")
                                                    .next_by_kanji(symbol.kanji),
                                            );
                                        }
                                    });
                                });
                            }
                        });
                });

                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.scope(|ui| {
                            ui.set_max_width(120.0);
                            if !is_furigana_revealed {
                                if ui.button("reveal reading").clicked() {
                                    self.furigana_revealed
                                        .replace(true)
                                        .expect("UI state is poisoned");
                                    is_furigana_revealed = true;
                                }
                            }
                        });
                    });
                });

                strip.strip(|builder| {
                    builder
                        .sizes(Size::remainder(), current_word.readings.len())
                        .horizontal(|mut strip| {
                            for reading in current_word.readings.iter() {
                                strip.cell(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let furigana = if is_furigana_revealed {
                                            reading.furigana.clone()
                                        } else {
                                            reading.furigana.chars().map(|_c| '何').collect()
                                        };
                                        if ui
                                            .button(egui::RichText::new(furigana).size(22.0))
                                            .clicked()
                                        {
                                            self.replace_word(
                                                DATABASE
                                                    .lock()
                                                    .expect("DB is poisoned!")
                                                    .next_by_reading(&reading),
                                            );
                                        }
                                    });
                                });
                            }
                        });
                });

                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.scope(|ui| {
                            ui.set_max_width(120.0);
                            if !is_translation_revealed {
                                if ui.button("reveal translation").clicked() {
                                    self.translation_revealed
                                        .replace(true)
                                        .expect("UI state is poisoned");
                                    is_translation_revealed = true;
                                }
                            }
                        });
                    });
                });

                strip.strip(|builder| {
                    builder
                        .sizes(Size::remainder(), current_word.translations.len())
                        .horizontal(|mut strip| {
                            for translation in current_word.translations.iter() {
                                strip.cell(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let translation = if is_translation_revealed {
                                            translation.clone()
                                        } else {
                                            translation.chars().map(|_c| '?').collect()
                                        };
                                        ui.label(egui::RichText::new(translation).size(22.0))
                                    });
                                });
                            }
                        });
                });
            });
    }
}

impl App {
    fn replace_word(&mut self, new_word: Word) {
        self.furigana_revealed
            .replace(false)
            .expect("UI state is poisoned");
        self.word.replace(new_word).expect("UI state is poisoned");
    }
}
