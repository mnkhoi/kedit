use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};
use crossterm::{Command, queue};
use std::io::{Error, Write, stdout};

#[derive(Clone, Copy, Default)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    pub const fn saturating_sub(&self, other: Self) -> Self {
        Self {
            row: self.row.saturating_sub(other.row),
            col: self.col.saturating_sub(other.col),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Terminal;

/// Represents the Terminal
/// Edge Case:
///     - Platform of `usize` < `ul6` -> Only spans over at most usize::MAX
///     or `u16::size` rows/columns
///     - Each size returned truncates to min(usize::MAX, u16::MAX)
///     - caret out of bound will be truncated
impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::show_caret()?;
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::clear_screen()?;
        Self::execute()
    }

    pub fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)
    }

    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)
    }

    pub fn print(output: &str) -> Result<(), Error> {
        Self::queue_command(Print(output))
    }

    pub fn print_row(row: usize, line_text: &str) -> Result<(), Error> {
        Self::move_caret_to(Position { col: 0, row })?;
        Self::clear_line()?;
        Self::print(line_text)?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::CurrentLine))
    }

    /// Returns: the current size of Terminal
    /// Edge Case:
    ///     - Systems with `usize` < `u16` -> Size represents the terminal size.
    ///       Any coordinate `x` truncates to size `usize` if `usize` < `x`< `u16`
    pub fn size() -> Result<Size, Error> {
        let size = size()?;
        #[allow(clippy::as_conversions)]
        Ok(Size {
            height: size.1 as usize,
            width: size.0 as usize,
        })
    }

    /// Moves caret to given Position
    /// Arguments:
    /// * `Position` - The `Position` to move the caret to. Will truncated to `u16::MAX` if larger
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }
}
