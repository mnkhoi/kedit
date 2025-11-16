use std::{fs::read_to_string, io::Error};

use super::line::Line;

#[derive(Default, Clone)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    // pub fn push(&mut self, token: &str) -> Result<(), Error> {
    //     self.lines.push(token.to_string());
    //     Ok(())
    // }
    //
    // pub fn size(&self) -> usize {
    //     self.lines.len()
    // }

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;

        let mut lines = Vec::new();

        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        Ok(Self { lines })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
