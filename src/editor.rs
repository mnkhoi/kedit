use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod command;
mod commandbar;
mod documentstatus;
mod line;
mod messagebar;
mod position;
mod size;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use command::{Command, InsertCommand, Mode};
use commandbar::CommandBar;
use documentstatus::DocumentStatus;
use line::Line;
use messagebar::MessageBar;
use position::Position;
use size::Size;
use statusbar::StatusBar;
use terminal::Terminal;
use uicomponent::UIComponent;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 2;

#[derive(Default, Eq, PartialEq)]
pub enum PromptType {
    #[default]
    None,
    Save,
    Search,
}

impl PromptType {
    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    mode: Mode,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    prompt_type: PromptType,
    terminal_size: Size,
    title: String,
    quit_times: u8,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.handle_resize_command(size);

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            editor.view.load(file_name);
        }

        editor
            .message_bar
            .update_message("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
        editor.refresh_status();

        Ok(editor)
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => {
                    self.evaluate_event(event);
                    let mut status = self.view.get_status();
                    status.mode = self.mode;
                    self.status_bar.update_status(status);
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }
        }
    }

    fn handle_resize_command(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });

        let bar_size = Size {
            height: 1,
            width: size.width,
        };

        self.message_bar.resize(bar_size);
        self.command_bar.resize(bar_size);
        self.status_bar.resize(bar_size);
    }

    fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.file_name);

        self.status_bar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    }

    fn in_prompt(&self) -> bool {
        !self.prompt_type.is_none()
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }

        let _ = Terminal::hide_caret();

        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        if self.in_prompt() {
            self.command_bar.render(bottom_bar_row);
        } else {
            self.message_bar.render(bottom_bar_row);
        }

        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }

        if self.terminal_size.height > 2 {
            self.view.render(0);
        }

        let new_caret_pos = if self.in_prompt() {
            Position {
                row: bottom_bar_row,
                col: self.command_bar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };

        let _ = Terminal::move_caret_to(new_caret_pos);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = Command::try_from(event, &self.mode) {
                self.process_command(command)
            }
        }
    }

    fn process_command(&mut self, command: Command) {
        if let Command::Resize(size) = command {
            self.handle_resize_command(size);
            return;
        }

        match self.prompt_type {
            PromptType::Save => self.process_command_save(command),
            PromptType::Search => self.process_command_search(command),
            PromptType::None => self.process_command_none(command),
        }
    }

    fn process_command_search(&mut self, command: Command) {
        match command {
            Command::Esc => {
                self.set_prompt(PromptType::None);
                self.mode = Mode::Normal;
                self.view.dismiss_search();
            }
            Command::Insert(InsertCommand::Enter) => {
                self.set_prompt(PromptType::None);
                self.mode = Mode::Normal;
                self.view.exit_search();
            }
            Command::Insert(insert) => {
                self.command_bar.handle_edit_command(insert);
                let query = self.command_bar.value();
                self.view.search(&query);
            }
            _ => {}
        }
    }

    fn process_command_save(&mut self, command: Command) {
        match command {
            Command::Esc => {
                self.set_prompt(PromptType::None);
                self.update_message("Save aborted.");
                self.mode = Mode::Normal;
            }
            Command::Insert(InsertCommand::Enter) => {
                let file_name = self.command_bar.value();
                self.save(Some(&file_name));
                self.set_prompt(PromptType::None);
                self.mode = Mode::Normal;
            }
            Command::Insert(insert) => self.command_bar.handle_edit_command(insert),
            _ => {}
        }
    }

    fn process_command_none(&mut self, command: Command) {
        if matches!(command, Command::Quit) {
            self.handle_quit();
            return;
        }
        self.reset_quit_times();

        match command {
            Command::Quit | Command::Resize(_) => {}
            Command::Esc => self.mode = Mode::Normal,
            Command::Search => {
                self.set_prompt(PromptType::Search);
                self.mode = Mode::Insert;
            }
            Command::Save => {
                self.handle_save();
                self.mode = Mode::Insert;
            }
            Command::Change(mode) => self.mode = mode,
            Command::Insert(insert) => self.view.handle_insert_command(insert),
            Command::Normal(normal) => self.view.handle_normal_command(normal),
        }
    }

    fn reset_quit_times(&mut self) {
        self.quit_times = 0;
    }

    fn update_message(&mut self, new_message: &str) {
        self.message_bar.update_message(new_message);
    }

    fn set_prompt(&mut self, prompt_type: PromptType) {
        match prompt_type {
            PromptType::None => self.message_bar.set_needs_redraw(true),
            PromptType::Save => self.command_bar.set_prompt("Save as: "),
            PromptType::Search => {
                self.view.enter_search();
                self.command_bar.set_prompt("Search (Esc to cancel): ");
            }
        }
        self.command_bar.clear_value();
        self.prompt_type = prompt_type;
    }

    fn handle_save(&mut self) {
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }

    fn save(&mut self, file_name: Option<&str>) {
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save()
        };
        if result.is_ok() {
            self.message_bar.update_message("File saved successfully.");
        } else {
            self.message_bar.update_message("Error writing file!");
        }
    }

    fn handle_quit(&mut self) {
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES {
            self.should_quit = true;
        } else if self.view.get_status().is_modified {
            self.message_bar.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                QUIT_TIMES - self.quit_times - 1
            ));
            self.quit_times += 1;
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye.\r\n");
        }
    }
}
