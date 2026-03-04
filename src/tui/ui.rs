//! # UI
//!
//! UI rendering for the TUI application.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
};

use super::app::{App, OptimizationMethod, Tab, View, WEIGHT_CLASSES};

/// Main draw function.
pub fn draw(frame: &mut Frame, app: &App) {
    match app.view {
        View::Rankings => draw_rankings(frame, app),
        View::FighterDetail => draw_fighter_detail(frame, app),
        View::Predictions => draw_predictions(frame, app),
        View::Optimize => draw_optimize(frame, app),
        View::BacktestConfig => {
            draw_optimize(frame, app);
            draw_backtest_config(frame, app);
        }
    }

    // Draw search overlay if in search mode
    if app.search_mode {
        draw_search_overlay(frame, app);
    }
}

/// Draws the rankings view.
fn draw_rankings(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header with tabs
            Constraint::Length(3), // Weight class selector
            Constraint::Min(10),   // Rankings table
            Constraint::Length(3), // Help bar
        ])
        .split(frame.area());

    // Header with tabs
    draw_header_with_tabs(frame, chunks[0], app.tab);

    // Weight class selector
    let (_, class_name) = WEIGHT_CLASSES[app.weight_class_index];
    let selector_text = format!(
        "< {} ({}/{}) >",
        class_name,
        app.weight_class_index + 1,
        WEIGHT_CLASSES.len()
    );

    let search_info = if !app.search_query.is_empty() {
        format!(" | Search: \"{}\"", app.search_query)
    } else {
        String::new()
    };

    let selector = Paragraph::new(format!("{}{}", selector_text, search_info))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Weight Class ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );
    frame.render_widget(selector, chunks[1]);

    // Rankings table
    let header_cells = [
        "#", "Fighter", "Rating", "Peak", "Record", "KO", "SUB", "DEC",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).bold()));
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let visible_height = chunks[2].height.saturating_sub(4) as usize;

    // Adjust scroll offset based on selection
    let scroll_offset = if app.selected_index >= app.scroll_offset + visible_height {
        app.selected_index.saturating_sub(visible_height - 1)
    } else if app.selected_index < app.scroll_offset {
        app.selected_index
    } else {
        app.scroll_offset
    };

    let rows: Vec<Row> = app
        .fighters
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, fighter)| {
            let is_selected = i == app.selected_index;
            let rank = i + 1;
            let record = format!("{}-{}", fighter.wins, fighter.losses);

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let rating_color = get_rating_color(fighter.rating);

            Row::new(vec![
                Cell::from(format!("{}", rank)),
                Cell::from(fighter.name.clone()),
                Cell::from(format!("{:.0}", fighter.rating))
                    .style(Style::default().fg(rating_color)),
                Cell::from(format!("{:.0}", fighter.max_rating)),
                Cell::from(record),
                Cell::from(format!("{}", fighter.ko_wins)),
                Cell::from(format!("{}", fighter.sub_wins)),
                Cell::from(format!("{}", fighter.dec_wins)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5), // Rank
            Constraint::Min(25),   // Name
            Constraint::Length(8), // Rating
            Constraint::Length(8), // Peak
            Constraint::Length(8), // Record
            Constraint::Length(5), // KO
            Constraint::Length(5), // SUB
            Constraint::Length(5), // DEC
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(format!(" Rankings ({} fighters) ", app.fighters.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(table, chunks[2]);

    // Scrollbar
    if app.fighters.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        let mut scrollbar_state =
            ScrollbarState::new(app.fighters.len()).position(app.selected_index);

        frame.render_stateful_widget(
            scrollbar,
            chunks[2].inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }

    // Help bar
    let help_text = vec![
        Span::styled(" q ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Quit  "),
        Span::styled(" Tab ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Next Tab  "),
        Span::styled(" ↑/↓ ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Navigate  "),
        Span::styled(" ←/→ ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Weight Class  "),
        Span::styled(" Enter ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Details  "),
        Span::styled(" / ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Search  "),
    ];

    let help = Paragraph::new(Line::from(help_text))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[3]);
}

/// Draws the fighter detail view.
fn draw_fighter_detail(frame: &mut Frame, app: &App) {
    let Some(ref fighter) = app.selected_fighter else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(10), // Fighter info
            Constraint::Min(10),    // Fight history
            Constraint::Length(3),  // Help bar
        ])
        .split(frame.area());

    // Fighter info card
    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // Left side: Name and rating
    let rating_color = get_rating_color(fighter.rating);
    let rating_bar = create_rating_bar(fighter.rating);

    let info_left = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            &fighter.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Rating: "),
            Span::styled(
                format!("{:.0}", fighter.rating),
                Style::default().fg(rating_color).bold(),
            ),
            Span::raw(format!(" (Peak: {:.0})", fighter.max_rating)),
        ]),
        Line::from(rating_bar),
        Line::from(""),
        Line::from(format!(
            "Weight Class: {}",
            fighter
                .weight_class
                .as_ref()
                .map(|w| format_weight_class(w))
                .unwrap_or_else(|| "Unknown".to_string())
        )),
    ])
    .block(
        Block::default()
            .title(" Fighter Profile ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(info_left, info_chunks[0]);

    // Right side: Record breakdown
    let total_fights = fighter.wins + fighter.losses;
    let win_pct = if total_fights > 0 {
        (fighter.wins as f64 / total_fights as f64) * 100.0
    } else {
        0.0
    };

    let info_right = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Record: "),
            Span::styled(
                format!("{}", fighter.wins),
                Style::default().fg(Color::Green).bold(),
            ),
            Span::raw(" - "),
            Span::styled(
                format!("{}", fighter.losses),
                Style::default().fg(Color::Red).bold(),
            ),
            Span::raw(format!(" ({:.1}%)", win_pct)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("KO/TKO: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", fighter.ko_wins)),
        ]),
        Line::from(vec![
            Span::styled("Submissions: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{}", fighter.sub_wins)),
        ]),
        Line::from(vec![
            Span::styled("Decisions: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", fighter.dec_wins)),
        ]),
    ])
    .block(
        Block::default()
            .title(" Record Breakdown ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );
    frame.render_widget(info_right, info_chunks[1]);

    // Fight history
    let header_cells = ["Result", "Opponent", "Method", "Date"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).bold()));
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .fight_history
        .iter()
        .map(|fight| {
            let result_style = if fight.is_win {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            Row::new(vec![
                Cell::from(if fight.is_win { "W" } else { "L" }).style(result_style),
                Cell::from(fight.opponent_name.clone()),
                Cell::from(fight.finish_method.clone()),
                Cell::from(fight.date.clone()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(25),
            Constraint::Length(20),
            Constraint::Length(12),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(format!(
                " Fight History ({} fights) ",
                app.fight_history.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(table, chunks[1]);

    // Help bar
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" q/Esc/Backspace ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Back to Rankings  "),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

/// Draws the search overlay.
fn draw_search_overlay(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());

    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("Search: {}_", app.search_query))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Search (Esc to cancel) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(search, area);
}

/// Draws the header with tab selection.
fn draw_header_with_tabs(frame: &mut Frame, area: Rect, current_tab: Tab) {
    let rankings_style = if current_tab == Tab::Rankings {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let predictions_style = if current_tab == Tab::Predictions {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let optimize_style = if current_tab == Tab::Optimize {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Paragraph::new(Text::from(vec![Line::from(vec![
        Span::styled(
            " COMBAT TRAINING ENGINE ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("[ Rankings ]", rankings_style),
        Span::raw("  "),
        Span::styled("[ Predictions ]", predictions_style),
        Span::raw("  "),
        Span::styled("[ Optimize ]", optimize_style),
    ])]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(header, area);
}

/// Draws the predictions view.
fn draw_predictions(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header with tabs
            Constraint::Length(3), // Event selector
            Constraint::Min(10),   // Fight predictions
            Constraint::Length(3), // Help bar
        ])
        .split(frame.area());

    // Header with tabs
    draw_header_with_tabs(frame, chunks[0], app.tab);

    // Loading or error state
    if app.predictions_loading {
        let loading = Paragraph::new("Loading predictions from ESPN...")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Upcoming Events ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(loading, chunks[1]);

        let empty = Paragraph::new("").block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, chunks[2]);
    } else if let Some(ref error) = app.predictions_error {
        let error_msg = Paragraph::new(error.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .title(" Error ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            );
        frame.render_widget(error_msg, chunks[1]);

        let empty = Paragraph::new("Press 'r' to retry")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, chunks[2]);
    } else if app.predictions.is_empty() {
        let no_events = Paragraph::new("No upcoming events found")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Upcoming Events ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(no_events, chunks[1]);

        let empty = Paragraph::new("").block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, chunks[2]);
    } else {
        // Event selector
        let event = &app.predictions[app.selected_event_index];
        let selector_text = format!(
            "< {} - {} ({}/{}) >",
            event.name,
            event.date,
            app.selected_event_index + 1,
            app.predictions.len()
        );

        let selector = Paragraph::new(selector_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Upcoming Events ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(selector, chunks[1]);

        // Fight predictions
        draw_fight_predictions(frame, chunks[2], event);
    }

    // Help bar
    let help_text = vec![
        Span::styled(" q ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Quit  "),
        Span::styled(" Tab ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Next Tab  "),
        Span::styled(" ←/→ ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Change Event  "),
        Span::styled(" r ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Refresh  "),
    ];

    let help = Paragraph::new(Line::from(help_text))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[3]);
}

/// Draws fight predictions for an event.
fn draw_fight_predictions(frame: &mut Frame, area: Rect, event: &crate::engine::EventPrediction) {
    let header_cells = [
        "Fighter 1",
        "Rating",
        "Win %",
        "vs",
        "Win %",
        "Rating",
        "Fighter 2",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).bold()));
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = event
        .fights
        .iter()
        .map(|fight| {
            let f1_rating = fight
                .fighter1_rating
                .map(|r| format!("{:.0}", r))
                .unwrap_or_else(|| "N/A".to_string());
            let f2_rating = fight
                .fighter2_rating
                .map(|r| format!("{:.0}", r))
                .unwrap_or_else(|| "N/A".to_string());

            let f1_prob = format!("{:.1}%", fight.fighter1_win_prob * 100.0);
            let f2_prob = format!("{:.1}%", fight.fighter2_win_prob * 100.0);

            // Determine favorite
            let (f1_style, f2_style) = if fight.fighter1_win_prob > fight.fighter2_win_prob {
                (
                    Style::default().fg(Color::Green).bold(),
                    Style::default().fg(Color::Red),
                )
            } else if fight.fighter2_win_prob > fight.fighter1_win_prob {
                (
                    Style::default().fg(Color::Red),
                    Style::default().fg(Color::Green).bold(),
                )
            } else {
                (Style::default(), Style::default())
            };

            // Add confidence indicator
            let confidence_style = if !fight.has_ratings {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(fight.fighter1_name.clone()).style(f1_style),
                Cell::from(f1_rating).style(confidence_style),
                Cell::from(f1_prob).style(f1_style),
                Cell::from("vs"),
                Cell::from(f2_prob).style(f2_style),
                Cell::from(f2_rating).style(confidence_style),
                Cell::from(fight.fighter2_name.clone()).style(f2_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),   // Fighter 1
            Constraint::Length(8), // Rating 1
            Constraint::Length(8), // Win % 1
            Constraint::Length(4), // vs
            Constraint::Length(8), // Win % 2
            Constraint::Length(8), // Rating 2
            Constraint::Min(20),   // Fighter 2
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(format!(
                " Fight Predictions ({} fights) ",
                event.fights.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(table, area);
}

/// Draws the optimize view.
fn draw_optimize(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header with tabs
            Constraint::Length(3), // Active config display
            Constraint::Length(3), // Method selector
            Constraint::Length(3), // Progress bar / status
            Constraint::Min(10),   // Results table
            Constraint::Length(3), // Help bar
        ])
        .split(frame.area());

    // Header with tabs
    draw_header_with_tabs(frame, chunks[0], app.tab);

    // Active config display
    let config_status = if app.active_config.is_custom() {
        format!("Active Config: {} ", app.active_config.summary())
    } else {
        "Active Config: Default ".to_string()
    };
    let resync_note = if app.needs_resync {
        "(needs re-sync)"
    } else {
        ""
    };
    let config_display = Paragraph::new(format!("{}{}", config_status, resync_note))
        .alignment(Alignment::Center)
        .style(if app.needs_resync {
            Style::default().fg(Color::Yellow)
        } else if app.active_config.is_custom() {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        })
        .block(
            Block::default()
                .title(" Current Elo Configuration ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(config_display, chunks[1]);

    // Method selector
    let method_text = match app.optimization_method {
        OptimizationMethod::Grid => format!("< {} > (240 configs)", app.optimization_method.name()),
        OptimizationMethod::Random => format!(
            "< {} > ({} samples, +/- to adjust)",
            app.optimization_method.name(),
            app.random_samples
        ),
    };

    let method_selector = Paragraph::new(method_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Optimization Method (m to toggle) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );
    frame.render_widget(method_selector, chunks[2]);

    // Progress bar / Status
    if app.optimization_running {
        let (current, total) = app.optimization_progress;
        let ratio = if total > 0 {
            current as f64 / total as f64
        } else {
            0.0
        };

        let config_info = app
            .optimization_current_config
            .as_ref()
            .map(|c| {
                format!(
                    " K={:.0} F={:.2} T={:.2} 5R={:.2}",
                    c.k_factor,
                    c.finish_multiplier,
                    c.title_fight_multiplier,
                    c.five_round_multiplier
                )
            })
            .unwrap_or_default();

        let progress = Gauge::default()
            .block(
                Block::default()
                    .title(format!(
                        " Progress: {}/{} ({:.1}%){}",
                        current,
                        total,
                        ratio * 100.0,
                        config_info
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .ratio(ratio);
        frame.render_widget(progress, chunks[3]);
    } else if let Some((ref message, is_success)) = app.optimization_message {
        let msg = Paragraph::new(message.as_str())
            .alignment(Alignment::Center)
            .style(if is_success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            })
            .block(
                Block::default()
                    .title(" Status ")
                    .borders(Borders::ALL)
                    .border_style(if is_success {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    }),
            );
        frame.render_widget(msg, chunks[3]);
    } else {
        let status = if app.optimization_results.is_empty() {
            "Press 'r' to run optimization"
        } else {
            "Optimization complete! Press 'a' to apply best config"
        };
        let status_bar = Paragraph::new(status).alignment(Alignment::Center).block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(status_bar, chunks[3]);
    }

    // Results table
    draw_optimization_results(frame, chunks[4], app);

    // Help bar
    let help_text = if app.optimization_running {
        vec![
            Span::styled(" q ", Style::default().bg(Color::DarkGray)),
            Span::raw(" Quit  "),
            Span::raw("(Optimization in progress...)"),
        ]
    } else {
        vec![
            Span::styled(" q ", Style::default().bg(Color::DarkGray)),
            Span::raw(" Quit  "),
            Span::styled(" Tab ", Style::default().bg(Color::DarkGray)),
            Span::raw(" Tab  "),
            Span::styled(" r ", Style::default().bg(Color::DarkGray)),
            Span::raw(" Run  "),
            Span::styled(" a ", Style::default().bg(Color::DarkGray)),
            Span::raw(" Apply  "),
            Span::styled(" Enter ", Style::default().bg(Color::DarkGray)),
            Span::raw(" Test  "),
            Span::styled(" e ", Style::default().bg(Color::DarkGray)),
            Span::raw(" Export  "),
        ]
    };

    let help = Paragraph::new(Line::from(help_text))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[5]);
}

/// Draws optimization results table.
fn draw_optimization_results(frame: &mut Frame, area: Rect, app: &App) {
    if app.optimization_results.is_empty() {
        let empty = Paragraph::new("No results yet. Run optimization to see results.")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let header_cells = [
        "#", "K-Factor", "Finish", "Title", "5-Round", "Log Loss", "Brier", "Accuracy",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).bold()));
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let visible_height = area.height.saturating_sub(4) as usize;

    // Adjust scroll offset based on selection
    let scroll_offset = if app.selected_result_index >= app.results_scroll_offset + visible_height {
        app.selected_result_index.saturating_sub(visible_height - 1)
    } else if app.selected_result_index < app.results_scroll_offset {
        app.selected_result_index
    } else {
        app.results_scroll_offset
    };

    let rows: Vec<Row> = app
        .optimization_results
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, result)| {
            let is_selected = i == app.selected_result_index;
            let is_best = i == 0;

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_best {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(format!("{:.0}", result.config.k_factor)),
                Cell::from(format!("{:.2}", result.config.finish_multiplier)),
                Cell::from(format!("{:.2}", result.config.title_fight_multiplier)),
                Cell::from(format!("{:.2}", result.config.five_round_multiplier)),
                Cell::from(format!("{:.4}", result.log_loss)),
                Cell::from(format!("{:.4}", result.brier_score)),
                Cell::from(format!("{:.2}%", result.accuracy * 100.0)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),  // #
            Constraint::Length(9),  // K-Factor
            Constraint::Length(8),  // Finish
            Constraint::Length(7),  // Title
            Constraint::Length(8),  // 5-Round
            Constraint::Length(10), // Log Loss
            Constraint::Length(8),  // Brier
            Constraint::Length(10), // Accuracy
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(format!(
                " Results ({} configs, sorted by log loss) ",
                app.optimization_results.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(table, area);

    // Scrollbar
    if app.optimization_results.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        let mut scrollbar_state =
            ScrollbarState::new(app.optimization_results.len()).position(app.selected_result_index);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// Draws the backtest configuration modal.
fn draw_backtest_config(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 16, frame.area());
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // K-Factor
            Constraint::Length(2), // Finish
            Constraint::Length(2), // Title
            Constraint::Length(2), // 5-Round
            Constraint::Length(1), // Spacer
            Constraint::Min(4),    // Results
            Constraint::Length(2), // Help
        ])
        .split(area);

    let block = Block::default()
        .title(" Custom Backtest ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    // Config fields
    let fields = [
        ("K-Factor:", format!("{:.1}", app.backtest_config.k_factor)),
        (
            "Finish Multiplier:",
            format!("{:.2}", app.backtest_config.finish_multiplier),
        ),
        (
            "Title Multiplier:",
            format!("{:.2}", app.backtest_config.title_fight_multiplier),
        ),
        (
            "5-Round Multiplier:",
            format!("{:.2}", app.backtest_config.five_round_multiplier),
        ),
    ];

    for (i, (label, value)) in fields.iter().enumerate() {
        let is_selected = i == app.config_field_index;
        let style = if is_selected {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default()
        };

        let arrow = if is_selected { ">" } else { " " };
        let text = format!("{} {:<20} < {} >", arrow, label, value);
        let para = Paragraph::new(text).style(style);
        frame.render_widget(para, chunks[i]);
    }

    // Results
    let result_text = if let Some(ref result) = app.backtest_result {
        vec![
            Line::from(vec![
                Span::raw("Log Loss:   "),
                Span::styled(
                    format!("{:.4}", result.log_loss),
                    Style::default().fg(Color::Cyan).bold(),
                ),
            ]),
            Line::from(vec![
                Span::raw("Brier:      "),
                Span::styled(
                    format!("{:.4}", result.brier_score),
                    Style::default().fg(Color::Cyan).bold(),
                ),
            ]),
            Line::from(vec![
                Span::raw("Accuracy:   "),
                Span::styled(
                    format!("{:.2}%", result.accuracy * 100.0),
                    Style::default().fg(Color::Cyan).bold(),
                ),
            ]),
            Line::from(vec![
                Span::raw("Fights:     "),
                Span::styled(
                    format!("{}", result.fights_processed),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]
    } else {
        vec![Line::from("Press Enter to run backtest")]
    };

    let results = Paragraph::new(result_text).block(
        Block::default()
            .title(" Results ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    frame.render_widget(results, chunks[5]);

    // Help
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓ ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Select  "),
        Span::styled(" ←/→ ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Adjust  "),
        Span::styled(" Enter ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Test  "),
        Span::styled(" s ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Save  "),
        Span::styled(" Esc ", Style::default().bg(Color::DarkGray)),
        Span::raw(" Back  "),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[6]);
}

/// Gets a color based on rating value.
fn get_rating_color(rating: f64) -> Color {
    if rating >= 1150.0 {
        Color::Rgb(255, 215, 0) // Gold
    } else if rating >= 1100.0 {
        Color::Green
    } else {
        Color::White
    }
}

/// Creates a visual rating bar.
fn create_rating_bar(rating: f64) -> Vec<Span<'static>> {
    let normalized = ((rating - 500.0) / 1000.0).clamp(0.0, 1.0);
    let filled = (normalized * 20.0) as usize;
    let empty = 20 - filled;

    vec![
        Span::styled(
            "█".repeat(filled),
            Style::default().fg(get_rating_color(rating)),
        ),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
    ]
}

/// Formats a weight class slug to a readable name.
fn format_weight_class(slug: &str) -> String {
    for (s, name) in WEIGHT_CLASSES {
        if *s == slug {
            return name.to_string();
        }
    }
    slug.replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Creates a centered rectangle.
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
