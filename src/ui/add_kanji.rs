//! A popup to add kanji

// use std::sync::{LazyLock, Mutex};

// use egui::{Layout, Popup, Response, ScrollArea};
// use egui_extras::{Size, StripBuilder};

// static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::new()));

// pub(crate) struct State {
//     partial_word: String,
//     furigana: Vec<char>,
//     translations: Vec<String>,
// }

// impl State {
//     fn new() -> Self {
//         Self {
//             partial_word: String::new(),
//             furigana: vec![],
//             translations: vec![],
//         }
//     }
// }

// pub(crate) fn popup(button: &Response) {
//     Popup::menu(button)
//         .gap(5.0)
//         .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
//         .show(|ui| {
//             let mut content = STATE.lock().expect("kanji adding state is poisoned");
//             ScrollArea::both().show(ui, |ui| {
//                 StripBuilder::new(ui)
//                     .sizes(Size::remainder(), 3)
//                     .vertical(|mut strip| {
//                         strip.cell(|ui| {
//                             ui.add(
//                                 egui::TextEdit::singleline(&mut content.partial_word)
//                                     .hint_text("漢字"),
//                             );
//                         });
//                         strip.cell(|ui| {
//                             if ui
//                                 .button(egui::include_image!("../../res/add.png"))
//                                 .clicked()
//                             {
//                                 todo!()
//                             }
//                         });
//                     });
//             });
//         });
// }
