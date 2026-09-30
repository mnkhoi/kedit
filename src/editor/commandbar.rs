use std::io::Error;

use super::{
    terminal::{Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct CommandBar {
    path_str: String,
    needs_redraw: bool,
    size: Size,
}

impl CommandBar {
    pub fn update_path(&mut self, new_path: String) {
        self.path_str = new_path;
        self.set_needs_redraw(true);
    }
    pub fn clear(&mut self) {
        self.path_str = String::from("");
        self.set_needs_redraw(true);
    }
}

impl UIComponent for CommandBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let message = format!(
            "Save as: {path:.remain$}",
            path = &self.path_str,
            remain = self.size.width.saturating_sub(9)
        );
        Terminal::print_row(origin_y, &message)
    }
}
