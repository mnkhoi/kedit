use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::terminal::Size;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug)]
pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}

#[derive(Debug)]
pub enum NormalCommand {
    Move(Direction),
}

#[derive(Debug)]
pub enum InsertCommand {
    Char(char),
}

#[derive(Debug)]
pub enum VisualCommand {
    None,
}

#[derive(Debug)]
pub enum EditorCommand {
    Normal(NormalCommand),
    Insert(InsertCommand),
    Visual(VisualCommand),

    // Global events through all the modes
    Resize(Size),
    Change(Mode),
    Esc,
    Quit,
}

impl EditorCommand {
    #[allow(clippy::as_conversions)]
    pub fn try_from(event: Event, mode: &Mode) -> Result<Self, String> {
        // println!("Command: {event:?}, mode: {mode:?}");
        match (&event, &mode) {
            (Event::Resize(width_u16, height_u16), _) => {
                let height = height_u16.clone() as usize;
                let width = width_u16.clone() as usize;

                Ok(Self::Resize(Size { height, width }))
            }
            (Event::Key(KeyEvent { code, .. }), _) if code.is_esc() => Ok(Self::Esc),
            (_, Mode::Normal) => Self::from_normal_command(event),
            (_, Mode::Visual) => Self::from_visual_command(event),
            (_, Mode::Insert) => Self::from_insert_command(event),
        }
    }

    fn from_normal_command(event: Event) -> Result<Self, String> {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match (code, modifiers) {
                (KeyCode::Char('i'), _) => Ok(Self::Change(Mode::Insert)),
                (KeyCode::Char('v'), _) => Ok(Self::Change(Mode::Visual)),
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    Ok(Self::Normal(NormalCommand::Move(Direction::Up)))
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    Ok(Self::Normal(NormalCommand::Move(Direction::Down)))
                }
                (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                    Ok(Self::Normal(NormalCommand::Move(Direction::Right)))
                }
                (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                    Ok(Self::Normal(NormalCommand::Move(Direction::Left)))
                }
                (KeyCode::PageDown, _) => {
                    Ok(Self::Normal(NormalCommand::Move(Direction::PageDown)))
                }
                (KeyCode::PageUp, _) => Ok(Self::Normal(NormalCommand::Move(Direction::PageUp))),
                (KeyCode::Home, _) => Ok(Self::Normal(NormalCommand::Move(Direction::Home))),
                (KeyCode::End, _) => Ok(Self::Normal(NormalCommand::Move(Direction::End))),
                _ => Err(format!("Key Code not supported: {code:?}")),
            }
        } else {
            Err(format!("Event not processed for normal command: {event:?}"))
        }
    }

    fn from_insert_command(event: Event) -> Result<Self, String> {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match (code, modifiers) {
                (KeyCode::Char(a), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    Ok(Self::Insert(InsertCommand::Char(a)))
                }
                _ => Err(format!(
                    "Other event not supported in insert mode: {event:?}"
                )),
            }
        } else {
            Err(format!("Event not processed for normal command: {event:?}"))
        }
    }

    // TODO: Temporary unused variables
    #[allow(unused_variables)]
    fn from_visual_command(event: Event) -> Result<Self, String> {
        Ok(Self::Visual(VisualCommand::None))
    }
}
