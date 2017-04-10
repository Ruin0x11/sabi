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

    pub fn message_lines(&self, canvas_width: usize) -> Vec<String> {
        let more_str = format!(" {}", MORE);
        let combined = self.combined_message();
        let mut wrapped_lines = textwrap::wrap(combined.as_str(), canvas_width);
        for line in wrapped_lines.iter_mut() {
            // TODO: Truncate with '...' instead, if the line is a single block
            // greater than the canvas width.
            line.truncate(canvas_width);
        }
        wrapped_lines
    }
}

impl<'a> World {
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
}
