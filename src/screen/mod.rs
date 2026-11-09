use crate::{NAME_VERSION, message};

pub mod add_kanji;
pub mod main;

/// common interface for screens
trait Screen {
    fn name(&self) -> &'static str;
    fn title(&self) -> String {
        let mut title = format!("{} - {}", NAME_VERSION, self.name(),);

        if let Some(msg) = message::current() {
            title.push_str(" | ");
            title.push_str(&msg);
        }

        title
    }
}
