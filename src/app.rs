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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    Words(usize),
    Time(u64), // Duration in seconds
}

impl std::fmt::Display for TestMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestMode::Words(n) => write!(f, "Words: {}", n),
            TestMode::Time(s) => write!(f, "Time: {}s", s),
        }
    }
}

pub struct App {
    pub running: bool,
    pub mode: AppMode,
    pub input: String,
    pub target_text: String,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub cursor_position: usize,
    // Settings
    pub test_mode: TestMode,
    pub include_punctuation: bool,
    pub include_numbers: bool,
    // Stats
    pub total_correct_strokes: usize,
    pub total_incorrect_strokes: usize,
    // Analytics
    pub wpm_history: Vec<(f64, f64)>, // (time, wpm)
    pub error_points: Vec<(f64, f64)>, // (time, wpm_at_error)
    pub last_wpm_sample: Option<Instant>,
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
            cursor_position: 0,
            test_mode: TestMode::Words(10),
            include_punctuation: false,
            include_numbers: false,
            total_correct_strokes: 0,
            total_incorrect_strokes: 0,
            wpm_history: Vec::new(),
            error_points: Vec::new(),
            last_wpm_sample: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        if self.mode == AppMode::Typing {
            // Sample WPM every 1 second
            if let Some(start) = self.start_time {
                let now = Instant::now();
                let elapsed = now.duration_since(start).as_secs_f64();
                
                // Only start sampling after at least 1 second has passed to avoid inflated initial WPM
                if elapsed >= 1.0 {
                    let should_sample = match self.last_wpm_sample {
                        None => true,
                        Some(last) => now.duration_since(last).as_secs_f64() >= 1.0,
                    };

                    if should_sample {
                        let current_wpm = self.calculate_wpm();
                        self.wpm_history.push((elapsed, current_wpm));
                        self.last_wpm_sample = Some(now);
                    }
                }
            }

            if let TestMode::Time(duration) = self.test_mode {
                if let Some(start) = self.start_time {
                    if start.elapsed().as_secs() >= duration {
                        self.end_time = Some(Instant::now());
                        // Capture final sample (only if at least 1 second has passed)
                        let elapsed = start.elapsed().as_secs_f64();
                        if elapsed >= 1.0 {
                            let current_wpm = self.calculate_wpm();
                            self.wpm_history.push((elapsed, current_wpm));
                        }
                        
                        self.mode = AppMode::Results;
                    }
                }
            }
        }
    }

    pub fn start_typing(&mut self) {
        let count = match self.test_mode {
            TestMode::Words(n) => n,
            TestMode::Time(_) => 100, // Generate enough words for time mode, can refill if needed
        };
        
        let words = words::get_random_words(count, self.include_punctuation, self.include_numbers);
        self.target_text = words.join(" ");
        self.input = String::new();
        self.mode = AppMode::Typing;
        self.start_time = None; // Don't start timer yet - wait for first keystroke
        self.end_time = None;
        self.cursor_position = 0;
        self.total_correct_strokes = 0;
        self.total_incorrect_strokes = 0;
        self.wpm_history = Vec::new();
        self.error_points = Vec::new();
        self.last_wpm_sample = None;
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
                KeyCode::Char('w') => self.cycle_word_mode(),
                KeyCode::Char('t') => self.cycle_time_mode(),
                KeyCode::Char('p') => self.include_punctuation = !self.include_punctuation,
                KeyCode::Char('n') => self.include_numbers = !self.include_numbers,
                _ => {}
            },
            AppMode::Typing => match key.code {
                KeyCode::Esc => {
                    self.mode = AppMode::Welcome;
                    self.start_time = None;
                }
                KeyCode::Char(c) => {
                    // Start timer on first keystroke
                    if self.start_time.is_none() {
                        self.start_time = Some(Instant::now());
                    }
                    
                    // Check if correct BEFORE updating input
                    let target_char = self.target_text.chars().nth(self.cursor_position);
                    if let Some(tc) = target_char {
                        if c == tc {
                            self.total_correct_strokes += 1;
                        } else {
                            self.total_incorrect_strokes += 1;
                            // Record error point
                            if let Some(start) = self.start_time {
                                let elapsed = start.elapsed().as_secs_f64();
                                let current_wpm = self.calculate_wpm();
                                self.error_points.push((elapsed, current_wpm));
                            }
                        }
                    } else {
                         // Typing beyond end of string counts as incorrect
                         self.total_incorrect_strokes += 1;
                         if let Some(start) = self.start_time {
                            let elapsed = start.elapsed().as_secs_f64();
                            let current_wpm = self.calculate_wpm();
                            self.error_points.push((elapsed, current_wpm));
                        }
                    }

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
        match self.test_mode {
            TestMode::Words(_) => {
                if self.input.len() >= self.target_text.len() {
                    self.end_time = Some(Instant::now());
                    // Capture final sample (only if at least 1 second has passed)
                    if let Some(start) = self.start_time {
                        let elapsed = start.elapsed().as_secs_f64();
                        if elapsed >= 1.0 {
                            let current_wpm = self.calculate_wpm();
                            self.wpm_history.push((elapsed, current_wpm));
                        }
                    }
                    self.mode = AppMode::Results;
                }
            }
            TestMode::Time(_) => {
                // In time mode, we don't end on completion, we might need to append more words if they type fast
                // For now, let's just assume 100 words is enough or end if they finish (unlikely for 100 words in short time)
                if self.input.len() >= self.target_text.len() {
                     self.end_time = Some(Instant::now());
                     // Capture final sample (only if at least 1 second has passed)
                     if let Some(start) = self.start_time {
                        let elapsed = start.elapsed().as_secs_f64();
                        if elapsed >= 1.0 {
                            let current_wpm = self.calculate_wpm();
                            self.wpm_history.push((elapsed, current_wpm));
                        }
                     }
                     self.mode = AppMode::Results;
                }
            }
        }
    }

    fn cycle_word_mode(&mut self) {
        self.test_mode = match self.test_mode {
            TestMode::Words(10) => TestMode::Words(25),
            TestMode::Words(25) => TestMode::Words(50),
            TestMode::Words(50) => TestMode::Words(100),
            _ => TestMode::Words(10),
        };
    }

    fn cycle_time_mode(&mut self) {
        self.test_mode = match self.test_mode {
            TestMode::Time(15) => TestMode::Time(30),
            TestMode::Time(30) => TestMode::Time(60),
            _ => TestMode::Time(15),
        };
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
        let total_strokes = self.total_correct_strokes + self.total_incorrect_strokes;
        if total_strokes == 0 {
            return 100.0;
        }
        (self.total_correct_strokes as f64 / total_strokes as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_accuracy_perfect() {
        let mut app = App::new();
        app.mode = AppMode::Typing; // Enable typing mode
        app.target_text = "hello world".to_string();
        // Simulate typing correctly
        for c in "hello world".chars() {
            app.handle_key_event(KeyEvent::from(KeyCode::Char(c)));
        }
        assert_eq!(app.calculate_accuracy(), 100.0);
    }

    #[test]
    fn test_calculate_accuracy_partial() {
        let mut app = App::new();
        app.mode = AppMode::Typing; // Enable typing mode
        app.target_text = "hello".to_string();
        
        // Type 'h', 'e', 'x' (wrong), 'l', 'l', 'o'
        // Correct: h, e, l, l, o (5)
        // Incorrect: x (1)
        // Total: 6
        // Accuracy: 5/6 * 100 = 83.33%
        
        app.handle_key_event(KeyEvent::from(KeyCode::Char('h')));
        app.handle_key_event(KeyEvent::from(KeyCode::Char('e')));
        app.handle_key_event(KeyEvent::from(KeyCode::Char('x'))); // Wrong
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace)); // Correct it
        app.handle_key_event(KeyEvent::from(KeyCode::Char('l')));
        app.handle_key_event(KeyEvent::from(KeyCode::Char('l')));
        app.handle_key_event(KeyEvent::from(KeyCode::Char('o')));

        let expected = (5.0 / 6.0) * 100.0;
        assert!((app.calculate_accuracy() - expected).abs() < 0.001);
    }

    #[test]
    fn test_calculate_accuracy_empty() {
        let mut app = App::new();
        app.mode = AppMode::Typing;
        app.target_text = "hello".to_string();
        assert_eq!(app.calculate_accuracy(), 100.0);
    }

    #[test]
    fn test_calculate_wpm() {
        let mut app = App::new();
        app.mode = AppMode::Typing; // Enable typing mode
        app.target_text = "hello world".to_string();
        // Simulate typing
        for c in "hello world".chars() {
             app.handle_key_event(KeyEvent::from(KeyCode::Char(c)));
        }
        
        let now = Instant::now();
        app.start_time = Some(now - Duration::from_secs(60)); // 1 minute ago
        app.end_time = Some(now);

        // 11 chars = 2.2 words
        // 2.2 words / 1 minute = 2.2 WPM
        assert!((app.calculate_wpm() - 2.2).abs() < 0.001);
    }
}
