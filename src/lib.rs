// #![feature(lock_value_accessors)]

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::{DefaultTerminal, layout::Position};
use strum::FromRepr;

pub const NAME_VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

mod database;
mod glyph;
mod message;
mod record;
mod screen;

#[repr(u32)]
#[derive(Default, FromRepr)]
enum Screen {
    #[default]
    Main = 1,
    AddKanji = 2,
}

#[derive(Default)]
pub struct App {
    current_screen: Screen,
    main_screen: screen::main::MainScreen,
    add_kanji_screen: screen::add_kanji::AddKanji,
    exit: bool,
}

impl App {
    /// main app cycle
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            self.draw(terminal)?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// frame re-drawer
    fn draw(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        terminal.draw(|frame| match self.current_screen {
            Screen::Main => frame.render_widget(&mut self.main_screen, frame.area()),
            Screen::AddKanji => frame.render_widget(&mut self.add_kanji_screen, frame.area()),
        })?;
        Ok(())
    }

    /// user event handler
    fn handle_events(&mut self) -> anyhow::Result<()> {
        // TODO: non-blocking poll instead
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
            _ => Ok(()), // TODO: log
        }
    }

    /// event handlers router
    fn handle_key_event(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        match (event.code, event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => self.exit = true,
            other_keypress => {
                // all numbers are reserved to switch screens
                if let KeyCode::Char(ch) = other_keypress.0
                    && ch.is_ascii_digit()
                {
                    let digit = ch.to_digit(10).unwrap();
                    if let Some(new_screen) = Screen::from_repr(digit) {
                        self.current_screen = new_screen;
                    }
                    return Ok(());
                }
                match self.current_screen {
                    Screen::Main => self.main_screen.handle_key_event(other_keypress),
                    Screen::AddKanji => self.add_kanji_screen.handle_key_event(other_keypress),
                }
            }
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<(), anyhow::Error> {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
            let position = Position::new(mouse_event.column, mouse_event.row);
            match self.current_screen {
                Screen::Main => self.main_screen.handle_mouse_event(position),
                Screen::AddKanji => self.add_kanji_screen.handle_mouse_event(position),
            }
        }

        Ok(())
    }
}
