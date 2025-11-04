use crossterm::event::{Event, Event::Key, KeyCode::Char, KeyEvent, KeyModifiers, read};
use std::io::Error;

mod terminal;

use terminal::{Position, Size, Terminal};

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub const fn default() -> Self {
        Self { should_quit: false }
    }

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
            self.evaluate_event(&event);
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n".to_string())?;
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor_to(Position { x: 0, y: 0 })?;
        }

        Terminal::show_cursor()?;
        Terminal::flush()?;
        Ok(())
    }

    fn draw_welcome_row() -> Result<(), Error> {
        let Size { height, width } = Terminal::size()?;
        let mut welcome_message = format!("{EDITOR_NAME} editor -- version {EDITOR_VERSION}");
        let len = welcome_message.len();

        let padding = (width as usize - len) / 2;

        let spaces = " ".repeat(padding - 1);

        welcome_message = format!("~{spaces}{welcome_message}");

        welcome_message.truncate(width as usize);

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
            Terminal::move_cursor_to(Position { x: 0, y: row })?;
            if row == height / 3 {
                Self::draw_welcome_row()?;
            } else {
                Self::draw_empty_row()?;
            }
            if row + 1 < height {
                Terminal::print("\r\n".to_string())?;
            }
        }

        Terminal::flush()
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            println!("Code: {code:?} Modifiers: {modifiers:?}\r");

            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }
}
