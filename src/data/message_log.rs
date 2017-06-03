use std::slice;
use std::collections::VecDeque;

const LINE_HEIGHT: usize =  16;

pub struct MessageLog {
    log: VecDeque<String>,
    next_line: bool,
    pub valid: bool,
}

impl MessageLog {
    pub fn new() -> Self {
        MessageLog {
            log: VecDeque::new(),
            next_line: true,
            valid: false,
        }
    }

    pub fn clear(&mut self) {
        self.log.clear();
    }

    pub fn append(&mut self, text: &str) {
        if self.next_line {
            self.log.push_front(String::new());
            self.next_line = false;
        }

        let mut current = match self.log.pop_front() {
            Some(line) => line,
            None       => String::new(),
        };

        current.push_str(text);
        current.push_str(" ");

        self.log.push_front(current);
    }

    pub fn next_line(&mut self) {
        self.next_line = true;
    }

    pub fn get_lines(&self, line_count: usize) -> Vec<String> {
        self.log.iter().take(line_count).cloned().collect()
    }
}
