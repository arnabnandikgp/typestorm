use crate::app::{App, AppMode, TestMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap, Table, Row, Cell},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
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
        AppMode::Welcome => "Press <Enter> to start | <w/t> change mode | <h> history | <q> quit".to_string(),
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
        AppMode::History => "Up/Down (j/k): Navigate | Enter: Details | q/Esc: Back".to_string(),
        AppMode::HistoryDetails => "Esc/q: Back to List".to_string(),
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

fn render_main(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default().borders(Borders::NONE).padding(ratatui::widgets::Padding::new(2, 2, 1, 1));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    match app.mode {
        AppMode::Welcome => {
             render_welcome(f, app, inner_area);
        }
        AppMode::Typing => {
             render_typing(f, app, inner_area);
        }
        AppMode::Results => {
            render_performance_view(
                f, 
                inner_area, 
                app.calculate_wpm(), 
                app.calculate_accuracy(), 
                &app.wpm_history,
                &app.error_points,
                true // is_new_result
            );
        }
        AppMode::History => {
            render_history_view(f, app, inner_area);
        }
        AppMode::HistoryDetails => {
            // Get selected history item
             // Visual index matches array index?? No, check history view implementation below.
             // We chose to show reversed list.
             // To simplify, let's just assume app.selected_history_index maps correctly to the item we want to show.
             // If we display list reversed, user pressing "down" (index++) moves visually down (older).
             // list[0] (top) = Newest.
             // app.history is Chronological (Oldest -> Newest).
             // So list[0] corresponds to app.history.last().
             // list[i] corresponds to app.history[len - 1 - i].
             
             let index = if app.selected_history_index < app.history.len() {
                 app.history.len() - 1 - app.selected_history_index
             } else {
                 0
             };
             
             if let Some(result) = app.history.get(index) {
                render_performance_view(
                    f, 
                    inner_area, 
                    result.wpm, 
                    result.accuracy, 
                    &result.wpm_history,
                    &result.error_points,
                    false
                );
             }
        }
    }
}

fn render_welcome(f: &mut Frame, app: &App, area: Rect) {
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
        Line::from(""),
        Line::from(Span::styled("[h] view history", Style::default().fg(Color::Magenta))),
    ];
    let p = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    // Center vertically
    let v_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(12),
            Constraint::Percentage(50),
        ])
        .split(area);
    
    f.render_widget(p, v_center[1]);
}

fn render_typing(f: &mut Frame, app: &App, area: Rect) {
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
    f.render_widget(p, area);
}

fn render_history_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Stats container
            Constraint::Min(1),     // List
        ])
        .split(area);

    // Stats Split
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[0]);

    // Stats Calculation
    let mut word_stats_map: std::collections::HashMap<String, (f64, f64, usize)> = std::collections::HashMap::new();
    let mut time_stats_map: std::collections::HashMap<String, (f64, f64, usize)> = std::collections::HashMap::new();

    for result in &app.history {
        let (map, key) = if result.mode.starts_with("Words") {
            (&mut word_stats_map, result.mode.clone())
        } else {
            (&mut time_stats_map, result.mode.clone())
        };
        
        // Normalize key to just the number for display if possible, or keep full mode string
        // The mode string is like "Words: 10" or "Time: 15s"
        
        let entry = map.entry(key).or_insert((0.0, 0.0, 0));
        entry.0 += result.wpm;
        entry.1 += result.accuracy;
        entry.2 += 1;
    }

    // Helper to render stats list
    fn render_stats_column(f: &mut Frame, map: std::collections::HashMap<String, (f64, f64, usize)>, title: &str, area: Rect) {
        let mut lines = Vec::new();
        let mut modes: Vec<&String> = map.keys().collect();
        modes.sort();

        for mode in modes {
            if let Some((total_wpm, total_acc, count)) = map.get(mode) {
                 let avg_wpm = total_wpm / *count as f64;
                 let avg_acc = total_acc / *count as f64;
                 lines.push(Line::from(vec![
                     Span::styled(format!("{:<15}", mode), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                     Span::raw(" | "),
                     Span::styled(format!("WPM: {:<5.1}", avg_wpm), Style::default().fg(Color::Yellow)),
                     Span::raw(" | "),
                     Span::styled(format!("Acc: {:.1}%", avg_acc), Style::default().fg(Color::Green)),
                     Span::raw(format!(" ({})", count)),
                 ]));
            }
        }
        
        if lines.is_empty() {
            lines.push(Line::from("No data."));
        }

        let block = Block::default().borders(Borders::ALL).title(title);
        let widget = Paragraph::new(lines).block(block);
        f.render_widget(widget, area);
    }

    render_stats_column(f, word_stats_map, "Word Tests", stats_chunks[0]);
    render_stats_column(f, time_stats_map, "Time Tests", stats_chunks[1]);


    // History List
    // We render Newest First (Reverse Order)
    let header_cells = ["Date", "Mode", "WPM", "Accuracy"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    let rows = app.history.iter().rev().map(|result| {
        let cells = vec![
            Cell::from(result.timestamp.format("%Y-%m-%d %H:%M").to_string()),
            Cell::from(result.mode.clone()),
            Cell::from(format!("{:.1}", result.wpm)),
            Cell::from(format!("{:.1}%", result.accuracy)),
        ];
        Row::new(cells)
    });
    
    let t = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
        ]
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Test History"))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">> ");
    
    f.render_stateful_widget(t, chunks[1], &mut app.history_state);
}

fn render_performance_view(
    f: &mut Frame, 
    area: Rect, 
    wpm: f64, 
    acc: f64, 
    wpm_history: &[(f64, f64)], 
    error_points: &[(f64, f64)],
    is_new_result: bool
) {
    let title = if is_new_result { "Test Complete!" } else { "Test Details" };
    
    let results_text = vec![
        Line::from(Span::styled(title, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
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
            Constraint::Length(6), // Results Text
            Constraint::Length(2), // Gap
            Constraint::Min(10),   // Graph area
        ])
        .split(area);
    
    f.render_widget(p, v_center[1]);

    // Render Graph
    use ratatui::{
        symbols,
        widgets::{Axis, Chart, Dataset, GraphType},
    };

    let raw_wpm_data: Vec<(f64, f64)> = wpm_history.to_vec();
    
    // Interpolate WPM data for smooth curve
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
        if let Some(last) = data.last() {
            smooth_data.push(*last);
        }
        
        smooth_data
    }

    let wpm_data = interpolate_data(&raw_wpm_data, 20); // 20 points between each sample
    
    let min_time = raw_wpm_data.first().map(|(t, _)| *t).unwrap_or(0.0);
    let max_time = raw_wpm_data.last().map(|(t, _)| *t).unwrap_or(60.0).max(1.0);
    
    // Process Error Data
    let bin_size = 1.5;
    let num_bins = (max_time / bin_size).ceil() as usize + 1;
    let mut error_bins = vec![0; num_bins];

    for (t, _) in error_points {
        let bin_index = (t / bin_size).floor() as usize;
        if bin_index < num_bins {
            error_bins[bin_index] += 1;
        }
    }

    let max_error_count = *error_bins.iter().max().unwrap_or(&0) as f64;
    let max_wpm = wpm_data.iter().map(|(_, w)| *w).fold(0.0, f64::max).max(10.0);

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

    let graph_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(6), 
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

    if max_error_count > 0.0 {
        let axis_area = graph_layout[1];
        let axis_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(axis_area);

        f.render_widget(
            Paragraph::new("Errs")
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)), 
            axis_split[0]
        );
        
        f.render_widget(
            Paragraph::new(format!("{:.0}", max_error_count))
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)), 
            axis_split[1]
        );

        f.render_widget(
            Paragraph::new("0")
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)), 
            axis_split[3]
        );
    }
}
