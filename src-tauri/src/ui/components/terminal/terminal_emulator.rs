pub struct TerminalEmulator {
    cols: u16,
    rows: u16,
    buffer: String,
}

impl TerminalEmulator {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            buffer: String::new(),
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    pub fn process_output(&mut self, data: &str) {
        self.buffer.push_str(data);
        // Keep only last 10000 lines
        let lines: Vec<&str> = self.buffer.lines().collect();
        if lines.len() > 10000 {
            self.buffer = lines[lines.len() - 10000..].join("\n");
        }
    }

    pub fn render(&self) -> &str {
        &self.buffer
    }
}