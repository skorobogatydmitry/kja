// #![feature(lock_value_accessors)]

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::{DefaultTerminal, layout::Position};

pub const NAME_VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

mod database;
mod glyph;
mod record;
mod screen;
mod ui;

#[derive(Default)]
enum Screen {
    #[default]
    Main,
}

#[derive(Default)]
pub struct App {
    current_screen: Screen,
    main_screen: screen::main::MainScreen,
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
        terminal.draw(|frame| {
            frame.render_widget(
                match self.current_screen {
                    Screen::Main => &mut self.main_screen,
                },
                frame.area(),
            );
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
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.exit = true
            }
            etc => match self.current_screen {
                Screen::Main => self.main_screen.handle_key_event(etc),
            },
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<(), anyhow::Error> {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
            let position = Position::new(mouse_event.column, mouse_event.row);
            match self.current_screen {
                Screen::Main => self.main_screen.handle_mouse_event(position),
            }
        }

        Ok(())
    }
}
