use crate::words;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::{Duration, Instant};

pub type AppResult<T> = Result<T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Welcome,
    Typing,
    Results,
}

pub struct App {
    pub running: bool,
    pub mode: AppMode,
    pub input: String,
    pub target_text: String,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub word_count: usize,
    pub cursor_position: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            mode: AppMode::Welcome,
            input: String::new(),
            target_text: String::new(),
            start_time: None,
            end_time: None,
            word_count: 10, // Default to 10 words
            cursor_position: 0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_typing(&mut self) {
        let words = words::get_random_words(self.word_count);
        self.target_text = words.join(" ");
        self.input = String::new();
        self.mode = AppMode::Typing;
        self.start_time = Some(Instant::now());
        self.end_time = None;
        self.cursor_position = 0;
    }

    pub fn handle_events(&mut self) -> AppResult<()> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key);
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.mode {
            AppMode::Welcome => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                KeyCode::Enter => self.start_typing(),
                _ => {}
            },
            AppMode::Typing => match key.code {
                KeyCode::Esc => {
                    self.mode = AppMode::Welcome;
                    self.start_time = None;
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                    self.cursor_position += 1;
                    self.check_completion();
                }
                KeyCode::Backspace => {
                    if !self.input.is_empty() {
                        self.input.pop();
                        self.cursor_position -= 1;
                    }
                }
                _ => {}
            },
            AppMode::Results => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                KeyCode::Enter => self.start_typing(),
                KeyCode::Char('r') => self.start_typing(),
                _ => {}
            },
        }
    }

    fn check_completion(&mut self) {
        if self.input.len() >= self.target_text.len() {
            self.end_time = Some(Instant::now());
            self.mode = AppMode::Results;
        }
    }
    
    pub fn calculate_wpm(&self) -> f64 {
        let duration = if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            end.duration_since(start)
        } else if let Some(start) = self.start_time {
             Instant::now().duration_since(start)
        } else {
            return 0.0;
        };

        let minutes = duration.as_secs_f64() / 60.0;
        if minutes == 0.0 {
            return 0.0;
        }
        
        let words = self.input.len() as f64 / 5.0;
        words / minutes
    }

    pub fn calculate_accuracy(&self) -> f64 {
        if self.input.is_empty() {
            return 100.0;
        }

        let correct_chars = self.input.chars()
            .zip(self.target_text.chars())
            .filter(|(a, b)| a == b)
            .count();

        (correct_chars as f64 / self.input.len() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_accuracy_perfect() {
        let mut app = App::new();
        app.target_text = "hello world".to_string();
        app.input = "hello world".to_string();
        assert_eq!(app.calculate_accuracy(), 100.0);
    }

    #[test]
    fn test_calculate_accuracy_partial() {
        let mut app = App::new();
        app.target_text = "hello world".to_string();
        app.input = "hello worlr".to_string(); // 'l' vs 'r' mismatch at index 10
        // 10 correct out of 11 total input
        let expected = (10.0 / 11.0) * 100.0;
        assert_eq!(app.calculate_accuracy(), expected);
    }

    #[test]
    fn test_calculate_accuracy_empty() {
        let mut app = App::new();
        app.target_text = "hello".to_string();
        app.input = "".to_string();
        assert_eq!(app.calculate_accuracy(), 100.0);
    }

    #[test]
    fn test_calculate_wpm() {
        let mut app = App::new();
        app.target_text = "hello world".to_string();
        app.input = "hello world".to_string(); // 11 chars = 2.2 words
        
        let now = Instant::now();
        app.start_time = Some(now - Duration::from_secs(60)); // 1 minute ago
        app.end_time = Some(now);

        // 2.2 words / 1 minute = 2.2 WPM
        assert!((app.calculate_wpm() - 2.2).abs() < 0.001);
    }
}
