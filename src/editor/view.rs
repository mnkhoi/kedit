use super::DocumentStatus;
use super::{
    Line, Mode, NAME, Position, Size, Terminal, VERSION,
    command::{Direction, InsertCommand, NormalCommand},
    uicomponent::UIComponent,
};
use std::{cmp::min, io::Error};

mod buffer;
mod fileinfo;
mod location;
mod searchinfo;

use buffer::Buffer;
use fileinfo::FileInfo;
use location::Location;
use searchinfo::SearchInfo;

#[derive(Default)]
pub struct View {
    scroll_offset: Position,
    text_location: Location,
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    search_info: Option<SearchInfo>,
}

impl View {
    // Start Region: Handle Editor Command

    pub const fn is_file_loaded(&self) -> bool {
        self.buffer.is_file_loaded()
    }

    pub fn handle_normal_command(&mut self, command: NormalCommand) {
        match command {
            NormalCommand::Move(direction) => self.move_text_location(&direction),
        }
    }
    // TODO: To be implemented when we have visual command
    // #[allow(unused_variables)]
    // fn handle_visual_command(&mut self, command: VisualCommand) {}

    pub fn handle_insert_command(&mut self, command: InsertCommand) {
        match command {
            InsertCommand::Move(_) => {}
            InsertCommand::Char(c) => {
                self.insert_char(c);
            }
            InsertCommand::Delete => {
                self.delete();
            }
            InsertCommand::Backspace => {
                self.backspace();
            }
            InsertCommand::Enter => self.insert_newline(),
        }
    }
    // End Region: Handle: Editor Command

    // Start Region: Misc

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.height(),
            current_line_index: self.text_location.line_index,
            is_modified: self.buffer.dirty,
            is_saving: self.buffer.saving,
            file_name: format!("{}", self.buffer.file_info),
            mode: Mode::Normal,
        }
    }

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.set_needs_redraw(true);
        }
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        self.buffer.save_as(file_name)
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return String::new();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        let remaining_width = width.saturating_sub(1);

        if remaining_width <= len {
            return "~".to_string();
        }
        format!("{:<1}{:^remaining_width$}", "~", welcome_message)
    }

    // End Region: Misc

    // Start Region: Rendering

    pub fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    // End Region: Rendering

    // Start Region: Scrolling

    fn scroll_vertically(&mut self, to: usize) {
        let Size { height, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;

            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);

            true
        } else {
            false
        };

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_horizontally(&mut self, to: usize) {
        let Size { width, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;

            true
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);

            true
        } else {
            false
        };

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();

        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    fn center_text_location(&mut self) {
        let Size { height, width } = self.size;
        let Position { row, col } = self.text_location_to_position();

        let vertical_mid = height.div_ceil(2);
        let horizontal_mid = width.div_ceil(2);

        self.scroll_offset.row = row.saturating_sub(vertical_mid);
        self.scroll_offset.col = col.saturating_sub(horizontal_mid);

        self.set_needs_redraw(true);
    }

    // End Region: Scrolling

    // Start Region: Positioning and Location

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_index;

        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });

        Position { col, row }
    }

    // End Region: Positioning and Location

    // Start Region: Text Location Movement

    #[allow(clippy::arithmetic_side_effects)]
    fn move_text_location(&mut self, direction: &Direction) {
        let Size { height, .. } = self.size;

        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }
        self.scroll_text_location_into_view();
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index -= 1;
        } else if self.text_location.line_index > 0 {
            // Vim like movement
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        if self.text_location.grapheme_index < line_width {
            self.text_location.grapheme_index += 1;
        } else {
            // Vim like movement
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                min(line.grapheme_count(), self.text_location.grapheme_index)
            });
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = min(self.text_location.line_index, self.buffer.height());
    }

    // End Region: Text Location Movement

    // Start Region: Text Mutation

    fn insert_char(&mut self, character: char) {
        let old_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        self.buffer.insert_char(character, self.text_location);

        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        let grapheme_delta = new_len.saturating_sub(old_len);

        if grapheme_delta > 0 {
            self.move_text_location(&Direction::Right);
        }

        self.needs_redraw = true;
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.move_text_location(&Direction::Right);
        self.needs_redraw = true;
    }

    fn delete(&mut self) {
        self.buffer.delete(self.text_location);
        self.needs_redraw = true;
    }

    fn backspace(&mut self) {
        if self.text_location.grapheme_index != 0 || self.text_location.line_index != 0 {
            self.move_text_location(&Direction::Left);
            self.delete();
        }
    }

    // End Region: Text Mutation

    // Start Region: Search

    pub fn enter_search(&mut self) {
        self.search_info = Some(SearchInfo {
            prev_location: self.text_location,
            prev_scroll_offset: self.scroll_offset,
            query: Line::default(),
        });
    }

    pub fn exit_search(&mut self) {
        self.search_info = None;
    }

    pub fn dismiss_search(&mut self) {
        if let Some(search_info) = &self.search_info {
            self.text_location = search_info.prev_location;
            self.scroll_offset = search_info.prev_scroll_offset;
            self.set_needs_redraw(true);
        }
        self.search_info = None;
    }

    pub fn search(&mut self, query: &str) {
        if let Some(search_info) = &mut self.search_info {
            search_info.query = Line::from(query);
        }
        self.search_from(self.text_location);
    }

    fn search_from(&mut self, from: Location) {
        if let Some(search_info) = self.search_info.as_ref() {
            let query = &search_info.query;
            if query.is_empty() {
                return;
            }
            if let Some(text_location) = self.buffer.search(query, from) {
                self.text_location = text_location;
                self.center_text_location();
            }
        }
    }

    pub fn search_next(&mut self) {
        let step_right;
        if let Some(search_info) = self.search_info.as_ref() {
            step_right = min(search_info.query.grapheme_count(), 1);
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Attempting to search_next without search_info");
            }
            #[cfg(not(debug_assertions))]
            {
                return;
            }
        }
        let location = Location {
            line_index: self.text_location.line_index,
            grapheme_index: self.text_location.grapheme_index.saturating_add(step_right),
        };
        self.search_from(location);
    }

    // End Region: Search
}

impl UIComponent for View {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_row: usize) -> Result<(), Error> {
        let Size { height, width } = self.size;
        let end_y = origin_row.saturating_add(height);

        #[allow(clippy::integer_division)]
        let top_third = height / 3;
        let scroll_top = self.scroll_offset.row;
        for current_row in origin_row..end_y {
            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);
            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                Self::render_line(current_row, &line.get_visible_graphemes(left..right));
            } else if current_row == top_third && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width));
            } else {
                Self::render_line(current_row, "~");
            }
        }
        Ok(())
    }
}
