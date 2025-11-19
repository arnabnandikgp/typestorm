use crate::app::{App, AppMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Main Content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_main(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, _app: &App, area: Rect) {
    let title = Paragraph::new("TypeStorm ⚡")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let info_text = match app.mode {
        AppMode::Welcome => "Press <Enter> to start | <q> to quit",
        AppMode::Typing => "Press <Esc> to cancel",
        AppMode::Results => "Press <Enter/r> to restart | <q> to quit",
    };

    let stats = if app.mode == AppMode::Typing {
        format!("WPM: {:.0} | Acc: {:.0}%", app.calculate_wpm(), app.calculate_accuracy())
    } else {
        String::new()
    };

    let footer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let info = Paragraph::new(info_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::TOP));
    
    let stats_widget = Paragraph::new(stats)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::TOP));

    f.render_widget(info, footer_layout[0]);
    f.render_widget(stats_widget, footer_layout[1]);
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::NONE).padding(ratatui::widgets::Padding::new(2, 2, 1, 1));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    match app.mode {
        AppMode::Welcome => {
            let welcome_text = vec![
                Line::from("Welcome to TypeStorm!"),
                Line::from(""),
                Line::from("Test your typing speed in the terminal."),
                Line::from(""),
                Line::from(Span::styled("Ready?", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Mode: "),
                    Span::styled(format!("{}", app.test_mode), Style::default().fg(Color::Yellow)),
                    Span::raw(" | "),
                    Span::raw("Punctuation: "),
                    Span::styled(if app.include_punctuation { "ON" } else { "OFF" }, 
                        if app.include_punctuation { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) }),
                    Span::raw(" | "),
                    Span::raw("Numbers: "),
                    Span::styled(if app.include_numbers { "ON" } else { "OFF" }, 
                        if app.include_numbers { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) }),
                ]),
                Line::from(""),
                Line::from(Span::styled("[w]ords [t]ime [p]unctuation [n]umbers", Style::default().fg(Color::DarkGray))),
            ];
            let p = Paragraph::new(welcome_text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            
            // Center vertically
            let v_center = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Length(10),
                    Constraint::Percentage(60),
                ])
                .split(inner_area);
            
            f.render_widget(p, v_center[1]);
        }
        AppMode::Typing => {
            let mut spans = Vec::new();
            let target_chars: Vec<char> = app.target_text.chars().collect();
            let input_chars: Vec<char> = app.input.chars().collect();

            for (i, &target_char) in target_chars.iter().enumerate() {
                if i < input_chars.len() {
                    let input_char = input_chars[i];
                    if input_char == target_char {
                        spans.push(Span::styled(target_char.to_string(), Style::default().fg(Color::Green)));
                    } else {
                        spans.push(Span::styled(target_char.to_string(), Style::default().fg(Color::Red).bg(Color::DarkGray)));
                    }
                } else if i == input_chars.len() {
                     // Cursor position - highlight the character we need to type
                     spans.push(Span::styled(target_char.to_string(), Style::default().fg(Color::Black).bg(Color::White)));
                } else {
                    spans.push(Span::styled(target_char.to_string(), Style::default().fg(Color::DarkGray)));
                }
            }

            let text = Text::from(Line::from(spans));
            let p = Paragraph::new(text)
                .wrap(Wrap { trim: true });
            f.render_widget(p, inner_area);
        }
        AppMode::Results => {
            let wpm = app.calculate_wpm();
            let acc = app.calculate_accuracy();
            
            let results_text = vec![
                Line::from(Span::styled("Test Complete!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![
                    Span::raw("WPM: "),
                    Span::styled(format!("{:.1}", wpm), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::raw("Accuracy: "),
                    Span::styled(format!("{:.1}%", acc), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]),
            ];
            
             let p = Paragraph::new(results_text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
             
             let v_center = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(10),
                    Constraint::Percentage(40),
                ])
                .split(inner_area);
            
            f.render_widget(p, v_center[1]);
        }
    }
}
