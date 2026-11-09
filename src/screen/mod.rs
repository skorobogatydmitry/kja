use ratatui::{style::Stylize, text::Line};

use crate::{NAME_VERSION, message};

pub mod add_word;
pub mod main;

/// common interface for screens
trait Screen {
    fn name(&self) -> &'static str;
    fn title(&self) -> Line<'_> {
        let mut title = Line::from(vec![
            NAME_VERSION.light_green(),
            " > ".into(),
            self.name().into(),
        ]);

        if let Some(msg) = message::current() {
            title.push_span(" | ");
            title.push_span(msg.red());
        }

        title
    }
}
