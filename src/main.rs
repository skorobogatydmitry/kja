use std::io::stdout;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};
use kja::App;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    execute!(stdout(), EnableMouseCapture).unwrap();

    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableMouseCapture);
        default_panic_hook(info);
    }));

    let result = ratatui::run(|terminal| App::default().run(terminal));

    let _ = execute!(stdout(), DisableMouseCapture);

    result
}
