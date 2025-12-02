use super::{DocumentStatus, Terminal, terminal::Size};

pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    margin_bottom: usize,
    width: usize,
    position_y: usize,
    is_visible: bool,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        let mut status_bar = Self {
            current_status: DocumentStatus::default(),
            margin_bottom,
            needs_redraw: true,
            width: size.width,
            position_y: size.height.saturating_sub(margin_bottom).saturating_sub(1),
            is_visible: false,
        };

        status_bar.resize(size);
        status_bar
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
        let mut position_y = 0;
        let mut is_visible = false;

        if let Some(result) = size
            .height
            .checked_sub(self.margin_bottom)
            .and_then(|result| result.checked_sub(1))
        {
            position_y = result;
            is_visible = true;
        }

        self.position_y = position_y;
        self.is_visible = is_visible;
        self.needs_redraw = true;
    }

    pub fn render(&mut self) {
        if !self.needs_redraw || !self.is_visible {
            return;
        }

        if let Ok(size) = Terminal::size() {
            let line_count = self.current_status.line_count_to_string();
            let modified_indicator = self.current_status.modified_indicator_to_string();
            let beginning = format!(
                "{} - {line_count} {modified_indicator}",
                self.current_status.file_name,
            );

            let position_indicator = self.current_status.position_indicator_to_string();
            let remainder_len = size.width.saturating_sub(beginning.len());
            let status = format!("{beginning}{position_indicator:>remainder_len$}");

            let to_print = if status.len() <= size.width {
                status
            } else {
                String::new()
            };

            let result = Terminal::print_inverted_row(self.position_y, &to_print);

            debug_assert!(result.is_ok(), "Failed to render status");
            self.needs_redraw = false;
        }
    }
}
