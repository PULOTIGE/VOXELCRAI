use crate::consciousness::core::ConsciousnessPulse;

pub struct ArchGuardLite {
    history: Vec<String>,
    max_entries: usize,
}

impl ArchGuardLite {
    pub fn new() -> Self {
        Self {
            history: Vec::with_capacity(128),
            max_entries: 512,
        }
    }

    pub fn record_pulse(&mut self, pulse: &ConsciousnessPulse) {
        let escaped = escape_html(&pulse.log);
        if is_safe(&escaped) {
            if self.history.len() >= self.max_entries {
                self.history.remove(0);
            }
            self.history.push(escaped);
        }
    }

    #[allow(dead_code)]
    pub fn last(&self) -> Option<&str> {
        self.history.last().map(|s| s.as_str())
    }
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn is_safe(input: &str) -> bool {
    input
        .chars()
        .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
}
