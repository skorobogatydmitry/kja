#![feature(lock_value_accessors)]

use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use eframe::CreationContext;
use egui::{FontData, FontDefinitions};
use egui_extras::{Size, StripBuilder};

use crate::record::Word;

pub const NAME_VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
static JP_FONT: &[u8] = include_bytes!("../res/KleeOne-Regular.ttf");

mod record;

pub struct App {
    word: Arc<Mutex<Word>>,
    word_revealed: Mutex<bool>,
}

impl App {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(cc: &CreationContext) -> Result<Box<dyn eframe::App>, Box<dyn Error + Send + Sync>> {
        // cc.egui_ctx.set_zoom_factor(Settings::get().zoom_factor);

        let mut fonts = FontDefinitions::default();
        let jp_font_name = String::from("KleeOne Regular");
        // Install my own font (maybe supporting non-latin characters):
        fonts.font_data.insert(
            jp_font_name.clone(),
            std::sync::Arc::new(FontData::from_static(JP_FONT)),
        );

        // Put my font as last fallback for monospace:
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push(jp_font_name);

        cc.egui_ctx.set_fonts(fonts);

        Ok(Box::new(Self {
            // TODO: smart / on-demand algo on choosing
            word: Arc::new(Mutex::new(Word::random_word())),
            word_revealed: Mutex::new(false),
        }))
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let current_word = self.word.get_cloned().expect("UI state is poisoned");
        let mut is_revealed = self
            .word_revealed
            .get_cloned()
            .expect("UI state it posoned");

        StripBuilder::new(ui)
            .size(Size::remainder()) // top strip of big cells
            .size(Size::exact(32.0)) // button row
            .size(Size::remainder()) // bottom strip of big cells
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .sizes(Size::remainder(), current_word.readings.len())
                        .horizontal(|mut strip| {
                            for symbol in current_word.readings.iter() {
                                strip.cell(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        if ui
                                            .button(egui::RichText::new(symbol.kanji).size(24.0))
                                            .clicked()
                                        {
                                            todo!("next word by kanji")
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
                            if !is_revealed {
                                if ui.button("reveal").clicked() {
                                    self.word_revealed
                                        .replace(true)
                                        .expect("UI state is poisoned");
                                    is_revealed = true;
                                }
                            }
                        });
                    });
                });

                strip.strip(|builder| {
                    builder
                        .sizes(Size::remainder(), current_word.readings.len())
                        .horizontal(|mut strip| {
                            for symbol in current_word.readings.iter() {
                                strip.cell(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let reading = if is_revealed {
                                            symbol.reading.clone()
                                        } else {
                                            symbol.reading.chars().map(|_c| '何').collect()
                                        };
                                        if ui
                                            .button(egui::RichText::new(reading).size(24.0))
                                            .clicked()
                                        {
                                            todo!("next word by reading")
                                        }
                                    });
                                });
                            }
                        });
                });
            });
    }
}
