use crate::app::{App, AppMode, TestMode};
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
        AppMode::Welcome => "Press <Enter> to start | <q> to quit".to_string(),
        AppMode::Typing => {
            if let TestMode::Time(duration) = app.test_mode {
                if let Some(start) = app.start_time {
                    let elapsed = start.elapsed().as_secs();
                    let remaining = duration.saturating_sub(elapsed);
                    format!("Time Remaining: {}s | Press <Esc> to cancel", remaining)
                } else {
                    // Timer hasn't started yet - show full duration
                    format!("Time Remaining: {}s | Press <Esc> to cancel", duration)
                }
            } else {
                "Press <Esc> to cancel".to_string()
            }
        },
        AppMode::Results => "Press <Enter/r> to restart | <q> to quit".to_string(),
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
                    Constraint::Length(2), // Top padding
                    Constraint::Length(6), // Results Text (4 lines + buffer)
                    Constraint::Length(2), // Gap
                    Constraint::Min(10),   // Graph area (take remaining space, min 10)
                ])
                .split(inner_area);
            
            f.render_widget(p, v_center[1]);

            // Render Graph
            use ratatui::{
                symbols,
                widgets::{Axis, Chart, Dataset, GraphType},
            };

            let raw_wpm_data: Vec<(f64, f64)> = app.wpm_history.clone();
            
            // Interpolate WPM data for smooth curve
            // Catmull-Rom Spline Interpolation
            fn interpolate_data(data: &[(f64, f64)], resolution: usize) -> Vec<(f64, f64)> {
                if data.len() < 2 {
                    return data.to_vec();
                }

                let mut smooth_data = Vec::new();
                
                for i in 0..data.len() - 1 {
                    let p0 = if i == 0 { data[0] } else { data[i - 1] };
                    let p1 = data[i];
                    let p2 = data[i + 1];
                    let p3 = if i + 2 < data.len() { data[i + 2] } else { p2 };

                    for t_step in 0..resolution {
                        let t = t_step as f64 / resolution as f64;
                        let t2 = t * t;
                        let t3 = t2 * t;

                        let x = 0.5 * (
                            (2.0 * p1.0) +
                            (-p0.0 + p2.0) * t +
                            (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2 +
                            (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3
                        );
                        
                        let y = 0.5 * (
                            (2.0 * p1.1) +
                            (-p0.1 + p2.1) * t +
                            (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2 +
                            (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3
                        );
                        
                        smooth_data.push((x, y));
                    }
                }
                // Add the very last point
                if let Some(last) = data.last() {
                    smooth_data.push(*last);
                }
                
                smooth_data
            }

            let wpm_data = interpolate_data(&raw_wpm_data, 20); // 20 points between each sample for smoother curve
            
            // Calculate time bounds from actual data
            let min_time = raw_wpm_data.first().map(|(t, _)| *t).unwrap_or(0.0);
            let max_time = raw_wpm_data.last().map(|(t, _)| *t).unwrap_or(60.0).max(1.0);
            
            // Process Error Data: Bin by 1.5s
            let bin_size = 1.5;
            let num_bins = (max_time / bin_size).ceil() as usize + 1;
            let mut error_bins = vec![0; num_bins];

            for (t, _) in &app.error_points {
                let bin_index = (t / bin_size).floor() as usize;
                if bin_index < num_bins {
                    error_bins[bin_index] += 1;
                }
            }

            let max_error_count = *error_bins.iter().max().unwrap_or(&0) as f64;
            let max_wpm = wpm_data.iter().map(|(_, w)| *w).fold(0.0, f64::max).max(10.0);

            // Normalize error counts to WPM scale
            let error_data: Vec<(f64, f64)> = error_bins.iter().enumerate()
                .filter(|(_, &count)| count > 0)
                .map(|(i, &count)| {
                    let time = i as f64 * bin_size;
                    let normalized_y = if max_error_count > 0.0 {
                        (count as f64 / max_error_count) * max_wpm
                    } else {
                        0.0
                    };
                    (time, normalized_y)
                })
                .collect();

            // Split graph area for Right Axis
            let graph_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(10),
                    Constraint::Length(6), // Space for Right Axis labels
                ])
                .split(v_center[3]);

            let datasets = vec![
                Dataset::default()
                    .name("WPM")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Cyan))
                    .graph_type(GraphType::Line)
                    .data(&wpm_data),
                Dataset::default()
                    .name("Errors")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Red))
                    .graph_type(GraphType::Scatter)
                    .data(&error_data),
            ];

            let chart = Chart::new(datasets)
                .block(Block::default().title("Performance").borders(Borders::ALL))
                .x_axis(
                    Axis::default()
                        .title("Time (s)")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([min_time, max_time])
                        .labels(vec![
                            Span::styled(format!("{:.0}", min_time), Style::default().add_modifier(Modifier::BOLD)),
                            Span::styled(format!("{:.0}", max_time), Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("WPM")
                        .style(Style::default().fg(Color::Cyan))
                        .bounds([0.0, max_wpm])
                        .labels(vec![
                            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                            Span::styled(format!("{:.0}", max_wpm), Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                );
            
            f.render_widget(chart, graph_layout[0]);

            // Render Right Axis (Errors) manually
            if max_error_count > 0.0 {
                // Adjust for borders of the chart
                // We want "Errs" to align with the top border/title area
                // We want the numbers to align with the grid lines (inside borders)
                
                let axis_area = graph_layout[1];
                
                // Layout for the whole right strip
                let axis_split = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // Row 0: Title "Errs" (Aligns with border)
                        Constraint::Length(1), // Row 1: Max Value (Aligns with top tick)
                        Constraint::Min(0),    // Spacer
                        Constraint::Length(1), // Row N-1: Min Value (Aligns with bottom tick)
                        Constraint::Length(1), // Row N: Bottom Border adjustment (empty)
                    ])
                    .split(axis_area);

                // Title
                f.render_widget(
                    Paragraph::new("Errs")
                        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Left), 
                    axis_split[0]
                );
                
                // Max Value
                f.render_widget(
                    Paragraph::new(format!("{:.0}", max_error_count))
                        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Left), 
                    axis_split[1]
                );

                // Min Value ("0") - Note: axis_split has 5 chunks. 
                // chunk[3] is the one before the last (bottom border).
                // The chart content ends at height-2 (1 for top border, 1 for bottom).
                // So "0" should be at index 3.
                f.render_widget(
                    Paragraph::new("0")
                        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Left), 
                    axis_split[3]
                );
            }
    }
    }
}
