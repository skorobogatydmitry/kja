use std::{env::args, io::stdout};

use anyhow::bail;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};
use kja::{App, DATABASE_FILE_PATH};
use log::info;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = args();
    args.next(); // skip the binary name
    if let Some(database_file_path) = args.next() {
        info!("loading database from {database_file_path}");
        match std::fs::exists(&database_file_path) {
            Ok(true) => match DATABASE_FILE_PATH.set(database_file_path.clone()) {
                Ok(()) => info!("Database file path is set to {database_file_path}"),
                Err(e) => bail!("Cannot set database file to {database_file_path}: {e}"),
            },
            Ok(false) => bail!("Database file `{database_file_path}' not found"),
            Err(e) => bail!("Cannot check database file `{database_file_path}': {e}"),
        }
    }

    execute!(stdout(), EnableMouseCapture).expect("cannot enable mouse capture");

    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableMouseCapture);
        default_panic_hook(info);
    }));

    let result = ratatui::run(|terminal| App::default().run(terminal));

    let _ = execute!(stdout(), DisableMouseCapture);

    result
}
