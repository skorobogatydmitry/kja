#![feature(lock_value_accessors)]

use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use eframe::CreationContext;
use egui::{FontData, FontDefinitions, Layout, Popup, ScrollArea};
use egui_extras::{Size, Strip, StripBuilder};

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
        //
        egui_extras::install_image_loaders(&cc.egui_ctx);

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

    fn top_bar(&mut self, strip: &mut Strip) {
        strip.strip(|builder| {
            builder
                .clip(true)
                .size(Size::remainder()) // TODO: dynamic calc
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            let info_popup_button = ui
                                .button(egui::include_image!("../res/question.png"))
                                .on_hover_text("info");
                            Popup::menu(&info_popup_button)
                                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                                .show(|ui| {
                                    ScrollArea::both()
                                        .show(ui, |ui| ui.label(include_str!("../README.md")));
                                });
                            if ui
                                .button(egui::include_image!("../res/cog.png"))
                                .on_hover_text("settings")
                                .clicked()
                            {
                                self.settings_menu(ui);
                            }
                        });
                    });
                });
        });
    }

    fn settings_menu(&self, ui: &mut egui::Ui) {
        todo!()
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
            .sizes(Size::remainder(), 4)
            .vertical(|mut strip| {
                self.top_bar(&mut strip);
                strip.strip(|builder| {
                    builder
                        .sizes(Size::remainder(), current_word.readings.len())
                        .horizontal(|mut strip| {
                            for symbol in current_word.readings.iter() {
                                strip.cell(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        if ui
                                            .button(
                                                egui::RichText::new(symbol.kanji)
                                                    .size(32.0)
                                                    .heading()
                                                    .strong(),
                                            )
                                            .highlight()
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
                if is_furigana_revealed {
                    strip.strip(|builder| {
                        builder
                            .sizes(Size::remainder(), current_word.readings.len())
                            .horizontal(|mut strip| {
                                for reading in current_word.readings.iter() {
                                    strip.cell(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            if ui
                                                .button(
                                                    egui::RichText::new(reading.furigana.as_str())
                                                        .size(22.0),
                                                )
                                                .clicked()
                                            {
                                                self.replace_word(
                                                    DATABASE
                                                        .lock()
                                                        .expect("DB is poisoned!")
                                                        .next_by_reading(reading),
                                                );
                                            }
                                        });
                                    });
                                }
                            });
                    });
                } else {
                    strip.cell(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.scope(|ui| {
                                ui.set_max_width(120.0);
                                if ui.button("reveal reading").clicked() {
                                    self.furigana_revealed
                                        .replace(true)
                                        .expect("UI state is poisoned");
                                    is_furigana_revealed = true;
                                }
                            });
                        });
                    });
                }

                if is_translation_revealed {
                    strip.strip(|builder| {
                        builder
                            .sizes(Size::remainder(), current_word.translations.len())
                            .horizontal(|mut strip| {
                                for translation in current_word.translations.iter() {
                                    strip.cell(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                egui::RichText::new(translation.as_str())
                                                    .size(22.0),
                                            )
                                        });
                                    });
                                }
                            });
                    });
                } else {
                    strip.cell(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.scope(|ui| {
                                ui.set_max_width(120.0);
                                if ui.button("reveal translation").clicked() {
                                    self.translation_revealed
                                        .replace(true)
                                        .expect("UI state is poisoned");
                                    is_translation_revealed = true;
                                }
                            });
                        });
                    });
                }
            });
    }
}

impl App {
    fn replace_word(&mut self, new_word: Word) {
        self.furigana_revealed
            .replace(false)
            .expect("UI state is poisoned");
        self.translation_revealed
            .replace(false)
            .expect("UI state is poisoned");
        self.word.replace(new_word).expect("UI state is poisoned");
    }
}
