use std::cell::LazyCell;

use ratatui::{style::Stylize, text::Line};
use strum::IntoEnumIterator;

use crate::{NAME_VERSION, message};

pub mod add_word;
pub mod main;

/// common interface for screens
pub(crate) trait Screen {
    const NAME: &'static str;
    const FOOTER: LazyCell<Line<'static>>;

    fn title(&self) -> Line<'_> {
        let mut title = Line::from(vec![
            NAME_VERSION.light_green(),
            " > ".into(),
            Self::NAME.into(),
        ]);

        if let Some(msg) = message::current() {
            title.push_span(" | ");
            title.push_span(msg.red());
        }

        title
    }

    fn footer(&self) -> Line<'_> {
        let mut footer = Line::default();
        for s in super::Screen::iter() {
            let name = s.name();
            footer.push_span(format!("<{}> ", s as u32));
            footer.push_span(name.light_cyan());
            footer.push_span(" | ");
        }

        footer.push_span("<arrows, mouse> ");
        footer.push_span("navigation".light_green());

        if !Self::FOOTER.iter().is_empty() {
            footer.push_span(" | ");
            footer.extend(Self::FOOTER.iter().cloned());
        }

        footer.push_span(" | ");
        footer.push_span("<^C> ");
        footer.push_span("exit".red());
        footer.push_span(" | ");
        footer.push_span("<h> ");
        footer.push_span("help".red());

        footer
    }
}
