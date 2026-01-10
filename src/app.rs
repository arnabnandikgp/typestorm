use crate::{words, history::{self, TestResult}};
use anyhow::Result;
use chrono::Local;
use ratatui::widgets::TableState;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::{Duration, Instant};

pub type AppResult<T> = Result<T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Welcome,
    Typing,
    Results,
    History,
    HistoryDetails,
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
    // History
    pub history: Vec<TestResult>,
    pub history_state: TableState,
    pub selected_history_index: usize,
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
            history: Vec::new(),
            history_state: TableState::default(),
            selected_history_index: 0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self::default();
        // Load history
        if let Ok(history) = history::load_history() {
            app.history = history;
        }
        app
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
                        
                        self.save_result();
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
                KeyCode::Char('h') => {
                    self.mode = AppMode::History;
                    self.history_state.select(Some(0));
                    self.selected_history_index = 0;
                }
                _ => {}
            },
            AppMode::History => match key.code {
                KeyCode::Esc => self.mode = AppMode::Welcome,
                KeyCode::Char('q') => self.mode = AppMode::Welcome,
                KeyCode::Up | KeyCode::Char('k') => {
                     if !self.history.is_empty() {
                        let i = match self.history_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.history.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.history_state.select(Some(i));
                        self.selected_history_index = i;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.history.is_empty() {
                        let i = match self.history_state.selected() {
                            Some(i) => {
                                if i >= self.history.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.history_state.select(Some(i));
                        self.selected_history_index = i;
                    }
                }
                KeyCode::Enter => {
                    if !self.history.is_empty() {
                        self.mode = AppMode::HistoryDetails;
                    }
                }
                _ => {}
            },
            AppMode::HistoryDetails => match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => self.mode = AppMode::History,
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
                        // Monkeytype-style: allow deletion of previous word only if it was incorrect
                        if let Some(last_char) = self.input.chars().last() {
                            if last_char == ' ' {
                                // Find the start of the word we just finished
                                let bytes = self.input.as_bytes();
                                let mut start_idx = 0;
                                if self.input.len() > 1 {
                                    for i in (0..self.input.len() - 1).rev() {
                                        if bytes[i] == b' ' {
                                            start_idx = i + 1;
                                            break;
                                        }
                                    }
                                }

                                let current_segment = &self.input[start_idx..];
                                let target_segment = self.target_text.get(start_idx..self.input.len()).unwrap_or("");

                                if current_segment == target_segment {
                                    // Word is correct, block backspace
                                    return;
                                }
                            }
                        }
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
                    self.save_result();
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
                     self.save_result();
                     self.mode = AppMode::Results;
                }
            }
        }
    }

    fn save_result(&mut self) {
        let result = TestResult {
            timestamp: Local::now(),
            mode: format!("{}", self.test_mode),
            wpm: self.calculate_wpm(),
            accuracy: self.calculate_accuracy(),
            wpm_history: self.wpm_history.clone(),
            error_points: self.error_points.clone(),
        };
        self.history.push(result);
        let _ = history::save_history(&self.history);
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

    #[test]
    fn test_refined_deletion() {
        let mut app = App::new();
        app.mode = AppMode::Typing;
        app.target_text = "hello world".to_string();
        
        // Case 1: Correct word + space -> Blocked
        for c in "hello ".chars() {
            app.handle_key_event(KeyEvent::from(KeyCode::Char(c)));
        }
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(app.input, "hello ", "Should NOT delete space if word is correct");
        
        // Case 2: Incorrect word + space -> Allowed
        app.input = "hellp ".to_string();
        app.cursor_position = 6;
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(app.input, "hellp", "Should allow deleting space if word is incorrect");
        
        // Case 3: Middle of word -> Allowed
        app.input = "hello".to_string();
        app.cursor_position = 5;
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(app.input, "hell", "Should allow deleting within word");

        // Case 4: Correct word after fixing it -> Blocked again
        app.input = "hell".to_string();
        app.cursor_position = 4;
        for c in "o ".chars() {
            app.handle_key_event(KeyEvent::from(KeyCode::Char(c)));
        }
        assert_eq!(app.input, "hello ");
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(app.input, "hello ", "Should block again once word is corrected");
    }

    #[test]
    fn test_accuracy_with_deletions() {
        let mut app = App::new();
        app.mode = AppMode::Typing;
        app.target_text = "hello".to_string();
        
        // Scenario 1: Mistake and correction
        // Type 'x' instead of 'h'
        app.handle_key_event(KeyEvent::from(KeyCode::Char('x'))); 
        assert_eq!(app.total_incorrect_strokes, 1);
        assert_eq!(app.total_correct_strokes, 0);
        
        // Backspace and type 'h'
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        app.handle_key_event(KeyEvent::from(KeyCode::Char('h')));
        assert_eq!(app.total_incorrect_strokes, 1);
        assert_eq!(app.total_correct_strokes, 1);
        
        // Accuracy should be 50% (1 correct / 2 total keystrokes)
        assert_eq!(app.calculate_accuracy(), 50.0);
        
        // Scenario 2: Deleting a correct character and re-typing it
        // Type 'e'
        app.handle_key_event(KeyEvent::from(KeyCode::Char('e')));
        assert_eq!(app.total_correct_strokes, 2);
        
        // Backspace 'e' and type 'e' again
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        app.handle_key_event(KeyEvent::from(KeyCode::Char('e')));
        
        // Currently, this INCREMENTS total_correct_strokes again
        assert_eq!(app.total_correct_strokes, 3);
        
        // Total strokes = 1 (x) + 1 (h) + 1 (e) + 1 (e) = 4
        // Accuracy = 3 / 4 = 75%
        assert_eq!(app.calculate_accuracy(), 75.0);
    }
}
