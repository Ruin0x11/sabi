use std::collections::VecDeque;

use textwrap;
use world::*;

const MORE: &'static str = "--More--";

pub struct Messages {
    messages: VecDeque<String>,
}

impl Messages {
    pub fn new() -> Self {
        Messages {
            messages: VecDeque::new(),
        }
    }
    pub fn add(&mut self, mes: String) {
        self.messages.push_back(mes);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    fn combined_message(&self) -> String {
        let mut combined = String::new();
        for mes in self.messages.iter() {
            combined.push_str(mes);
        }
        combined
    }

    /// Returns the lines to be printed by the canvas, separated by "--More--",
    /// based on the canvas width. Used for graphics backends with a small
    /// canvas size.
    pub fn message_lines(&self, canvas_width: usize) -> Vec<String> {
        let line_length = canvas_width - MORE.len() - 1;
        let more_str = format!(" {}", MORE);
        let combined = self.combined_message();
        let mut wrapped_lines = textwrap::wrap(combined.as_str(), line_length as usize);
        for line in wrapped_lines.iter_mut() {
            // TODO: Truncate with '...' instead, if the line is a single block
            // greater than the canvas width.
            line.truncate(canvas_width);
        }
        if wrapped_lines.len() > 1 {
            let truncated_count = wrapped_lines.len() - 1;
            // Don't place '--More--' on the last line.
            for line in wrapped_lines.iter_mut().take(truncated_count) {
                line.push_str(more_str.as_str());
            }
        }
        wrapped_lines
    }
}

impl World {
    pub fn message(&self, mes: String) {
        let true_mes = format!("{} ", mes);
        self.messages.borrow_mut().add(true_mes);
    }

    pub fn pop_messages(&self, width: usize) -> Vec<String> {
        let mes_vec = self.messages.borrow_mut().message_lines(width);
        debug!(self.logger, "Popped messages, length: {}", mes_vec.len());
        self.messages.borrow_mut().clear();
        mes_vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_WIDTH: usize = 80;

    #[test]
    fn test_single_line() {
        let mut messages = Messages::new();
        messages.add("Welcome, traveller!".to_string());
        assert_eq!(messages.message_lines(TEST_WIDTH), vec!["Welcome, traveller!".to_string()]);
    }

    #[test]
    fn test_two_lines() {
        let mut messages = Messages::new();
        messages.add("The putit hits! The putit hits! The putit hits! The putit hits! The putit hits! The putit hits! The putit hits!".to_string());
        assert_eq!(messages.message_lines(TEST_WIDTH),
                   vec!["The putit hits! The putit hits! The putit hits! The putit hits! The --More--".to_string(),
                        "putit hits! The putit hits! The putit hits!".to_string()]);
    }

    #[test]
    fn test_big_line() {
        let mut messages = Messages::new();
        let big = "O".repeat(TEST_WIDTH + 4);
        messages.add(big);
        assert_eq!(messages.message_lines(TEST_WIDTH),
                   vec!["O".repeat(TEST_WIDTH)]);
    }
}
