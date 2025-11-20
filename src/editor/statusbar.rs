use super::DocumentStatus;
use super::terminal::{Size, Terminal};

pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    margin_bottom: usize,
    width: usize,
    position_y: usize,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let Size { height, width } = Terminal::size().unwrap_or_default();
        Self {
            current_status: DocumentStatus::default(),
            margin_bottom,
            needs_redraw: true,
            width,
            position_y: height.saturating_sub(margin_bottom).saturating_sub(1),
        }
    }

    pub fn update_status(&mut self, updated_status: DocumentStatus) {
        if updated_status == self.current_status {
            return;
        }

        self.current_status = updated_status;
        self.needs_redraw = true;
    }

    pub fn resize(&mut self, size: Size) {
        self.width = size.width;
        self.position_y = size
            .height
            .saturating_sub(self.margin_bottom)
            .saturating_sub(1);
        self.needs_redraw = true;
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }
        let mut status = format!("{:?}", self.current_status);
        status.truncate(self.width);
        let result = Terminal::print_row(self.position_y, &status);

        debug_assert!(result.is_ok(), "Failed to render status bar.");
        self.needs_redraw = false;
    }
}
