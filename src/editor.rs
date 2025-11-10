use core::cmp::min;
use crossterm::event::{
    Event::{self, Key},
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read,
};
use std::io::Error;

mod terminal;

use terminal::{Position, Size, Terminal};

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Default)]
struct Location {
    x: usize,
    y: usize,
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    location: Location,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event)?;
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n".to_string())?;
        } else {
            Self::draw_rows()?;
            Terminal::move_caret_to(Position {
                col: self.location.x,
                row: self.location.y,
            })?;
        }

        Terminal::show_caret()?;
        Terminal::flush()?;
        Ok(())
    }

    fn draw_welcome_row() -> Result<(), Error> {
        let Size { width, .. } = Terminal::size()?;
        let mut welcome_message = format!("{EDITOR_NAME} editor -- version {EDITOR_VERSION}");
        let len = welcome_message.len();

        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len)) / 2;

        let spaces = " ".repeat(padding.saturating_sub(1));

        welcome_message = format!("~{spaces}{welcome_message}");

        welcome_message.truncate(width);

        Terminal::print(welcome_message)?;

        Ok(())
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }

    fn draw_rows() -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;

        for row in 0..height {
            Terminal::clear_line()?;
            Terminal::move_caret_to(Position { col: 0, row })?;
            #[allow(clippy::integer_division)]
            if row == height / 3 {
                Self::draw_welcome_row()?;
            } else {
                Self::draw_empty_row()?;
            }
            if row.saturating_add(1) < height {
                Terminal::print("\r\n".to_string())?;
            }
        }

        Terminal::flush()
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        if let Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            println!("Code: {code:?} Modifiers: {modifiers:?}\r");

            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Char('h')
                | KeyCode::Char('l')
                | KeyCode::Char('j')
                | KeyCode::Char('k')
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::End
                | KeyCode::Home => {
                    self.move_point(*code)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn move_point(&mut self, key_code: KeyCode) -> Result<(), Error> {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size()?;
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => y = y.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                y = min(y.saturating_add(1), height.saturating_sub(1))
            }
            KeyCode::Left | KeyCode::Char('h') => x = x.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => {
                x = min(x.saturating_add(1), width.saturating_sub(1))
            }
            KeyCode::PageUp => y = 0,
            KeyCode::PageDown => y = height.saturating_sub(1),
            KeyCode::Home => x = 0,
            KeyCode::End => x = width.saturating_sub(1),
            _ => (),
        }
        self.location = Location { x, y };
        Ok(())
    }
}
