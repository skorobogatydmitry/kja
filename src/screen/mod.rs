
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Widget},
};
use strum::IntoEnumIterator;

use crate::{NAME_VERSION, message};

pub mod add_word;
pub mod main;

/// common interface for screens
pub(crate) trait Screen {
    fn name(&self) -> &'static str;
    fn custom_footer(&self) -> Line<'static>;
    fn render_body(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer);

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

        let custom_footer = self.custom_footer();
        if !custom_footer.iter().is_empty() {
            footer.push_span(" | ");
            footer.extend(custom_footer.iter().cloned());
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

impl Widget for &mut dyn Screen {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // outer block
        Block::bordered()
            .title(self.title())
            .title_bottom(self.footer())
            .render(area, buf);

        self.render_body(area, buf);
    }
}
