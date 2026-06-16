use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::config::{self, Config, PROFILE_FIELDS};
use crate::{ACCENT, MUTED};

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Left,
    Right,
}

enum InputState {
    None,
    Editing { buffer: String, cursor: usize },
    Creating { buffer: String, cursor: usize },
}

pub struct App {
    config: Config,
    names: Vec<String>,
    profile_list: ListState,
    focus: Focus,
    field_idx: usize,
    input: InputState,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut names: Vec<String> = config.profiles.keys().cloned().collect();
        names.sort();
        let mut profile_list = ListState::default();
        if !names.is_empty() {
            profile_list.select(Some(0));
        }
        Self {
            config,
            names,
            profile_list,
            focus: Focus::Left,
            field_idx: 0,
            input: InputState::None,
        }
    }

    fn selected_index(&self) -> usize {
        self.profile_list.selected().unwrap_or(0)
    }

    fn selected_name(&self) -> Option<&str> {
        self.names.get(self.selected_index()).map(|s| s.as_str())
    }

    fn profile_next(&mut self) {
        if self.names.is_empty() {
            return;
        }
        let i = (self.selected_index() + 1) % self.names.len();
        self.profile_list.select(Some(i));
    }

    fn profile_prev(&mut self) {
        if self.names.is_empty() {
            return;
        }
        let i = if self.selected_index() == 0 {
            self.names.len() - 1
        } else {
            self.selected_index() - 1
        };
        self.profile_list.select(Some(i));
    }

    fn field_list_idx(&self) -> usize {
        match self.field_idx {
            0 => 1,  // description
            1 => 3,  // default
            2 => 4,  // small_fast
            3 => 5,  // default_haiku
            4 => 6,  // default_sonnet
            5 => 7,  // default_opus
            6 => 9,  // base_url
            7 => 10, // env_key
            _ => 1,
        }
    }

    fn field_next(&mut self) {
        self.field_idx = (self.field_idx + 1) % PROFILE_FIELDS.len();
    }

    fn field_prev(&mut self) {
        self.field_idx = if self.field_idx == 0 {
            PROFILE_FIELDS.len() - 1
        } else {
            self.field_idx - 1
        };
    }
}

fn footer_span(key: &str, _desc: &str) -> Span<'static> {
    Span::styled(
        format!(" {key} "),
        Style::default()
            .fg(Color::Black)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD),
    )
}

/// Build the field list items (with section headers) for the right panel.
fn build_field_items<'a>(profile: &'a config::Profile, field_idx: usize, input: &'a InputState) -> Vec<ListItem<'a>> {
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_section: Option<&str> = None;

    for (i, field) in PROFILE_FIELDS.iter().enumerate() {
        // Insert section header if section changed
        if current_section != Some(field.section) {
            current_section = Some(field.section);
            let header = Span::styled(
                format!(" {} ", field.section),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            );
            items.push(ListItem::new(Line::from(header)));
        }

        let value = (field.get)(profile);
        let value_str: String = value.clone().unwrap_or_default();

        let line = match input {
            InputState::Editing { buffer, cursor } if i == field_idx => {
                render_input_line(field.label, buffer, *cursor)
            }
            _ => {
                let val_style = if value.is_some() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(MUTED)
                };
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(field.label.to_string(), Style::default().fg(MUTED)),
                    Span::raw("  "),
                    Span::styled(value_str.clone(), val_style),
                ])
            }
        };
        items.push(ListItem::new(line));
    }

    items
}

fn render_input_line(label: &str, buffer: &str, cursor: usize) -> Line<'static> {
    use std::fmt::Write;
    let cursor = cursor.min(buffer.len());
    let mut display = String::with_capacity(buffer.len() + 1);
    write!(display, "{}█{}", &buffer[..cursor], &buffer[cursor..]).unwrap();

    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(label.to_string(), Style::default().fg(MUTED)),
        Span::raw("  "),
        Span::styled(display, Style::default().fg(Color::Yellow)),
    ])
}

pub fn run(mut app: App) -> (App, Option<String>) {
    let mut terminal = ratatui::init();

    loop {
        if let Err(e) = terminal.draw(|frame| {
            let area = frame.area();

            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(area);

            // ── header ──
            let header = Paragraph::new(Line::from(vec![
                Span::styled("cc", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled("  profile manager for Claude Code", Style::default().fg(MUTED)),
            ]));
            frame.render_widget(
                header.block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(MUTED)),
                ),
                v[0],
            );

            // ── body ──
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .split(v[1]);

            // ── left: profile list ──
            let profile_items: Vec<ListItem> = app
                .names
                .iter()
                .map(|name| {
                    let desc = app.config.profiles[name]
                        .description
                        .as_deref()
                        .unwrap_or("");
                    if desc.is_empty() {
                        ListItem::new(name.as_str())
                    } else {
                        ListItem::new(vec![
                            Line::from(name.as_str()),
                            Line::from(Span::styled(desc, Style::default().fg(MUTED))),
                        ])
                    }
                })
                .collect();

            let count = app.names.len();
            let border_style = if app.focus == Focus::Left {
                Style::default().fg(ACCENT)
            } else {
                Style::default().fg(MUTED)
            };
            let list = List::new(profile_items)
                .block(
                    Block::bordered()
                        .border_style(border_style)
                        .title(Span::styled(" Profiles ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
                        .title_bottom(Span::styled(format!(" {count} profiles "), Style::default().fg(MUTED))),
                )
                .highlight_style(Style::default().fg(Color::Black).bg(ACCENT))
                .highlight_symbol(" ▸");

            frame.render_stateful_widget(list, body[0], &mut app.profile_list);

            // ── right: field editor or create form ──
            let right_border = if app.focus == Focus::Right {
                Style::default().fg(ACCENT)
            } else {
                Style::default().fg(MUTED)
            };

            if let InputState::Creating { buffer, cursor } = &app.input {
                let c = (*cursor).min(buffer.len());
                let mut display = buffer.clone();
                display.insert(c, '█');
                let create_text = Paragraph::new(Line::from(vec![
                    Span::raw("\n"),
                    Span::raw("  "),
                    Span::styled("name: ", Style::default().fg(MUTED)),
                    Span::styled(display, Style::default().fg(Color::Yellow)),
                    Span::raw("\n\n  "),
                    Span::styled("Enter", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
                    Span::styled(" to create, ", Style::default().fg(MUTED)),
                    Span::styled("Esc", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
                    Span::styled(" to cancel", Style::default().fg(MUTED)),
                ]))
                .block(
                    Block::bordered()
                        .border_style(right_border)
                        .title(Span::styled(" New Profile ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))),
                );
                frame.render_widget(create_text, body[1]);
            } else {
                let selected = app.selected_name().unwrap_or("");
                let profile = app.config.profiles.get(selected);
                let field_items = if let Some(profile) = profile {
                    build_field_items(profile, app.field_idx, &app.input)
                } else {
                    vec![ListItem::new("")]
                };

                let mut field_list = List::new(field_items)
                    .block(
                        Block::bordered()
                            .border_style(right_border)
                            .title(Span::styled(
                                format!(" {} ", selected),
                                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                            )),
                    )
                    .highlight_style(Style::default().fg(Color::Black).bg(ACCENT));

                if app.focus == Focus::Right {
                    field_list = field_list.highlight_symbol(" ▸");
                }

                let mut field_state = ListState::default();
                if app.focus == Focus::Right {
                    field_state.select(Some(app.field_list_idx()));
                }
                frame.render_stateful_widget(field_list, body[1], &mut field_state);
            }

            // ── footer ──
            let footer = match (&app.input, app.focus) {
                (InputState::Creating { .. }, _) => Line::from(vec![
                    footer_span("Enter", "create"),
                    Span::raw(" "),
                    footer_span("Esc", "cancel"),
                ]),
                (InputState::Editing { .. }, _) => Line::from(vec![
                    footer_span("Enter", "confirm"),
                    Span::raw(" "),
                    footer_span("Esc", "cancel"),
                ]),
                (InputState::None, Focus::Left) => Line::from(vec![
                    footer_span("↑↓/jk", "navigate"),
                    Span::raw(" "),
                    footer_span("n", "new"),
                    Span::raw(" "),
                    footer_span("→", "edit"),
                    Span::raw(" "),
                    footer_span("Enter", "select"),
                    Span::raw(" "),
                    footer_span("q/ESC", "quit"),
                ]),
                (InputState::None, Focus::Right) => Line::from(vec![
                    footer_span("↑↓/jk", "navigate"),
                    Span::raw(" "),
                    footer_span("←", "back"),
                    Span::raw(" "),
                    footer_span("Enter", "edit"),
                    Span::raw(" "),
                    footer_span("q/ESC", "quit"),
                ]),
            };
            frame.render_widget(Paragraph::new(footer).centered(), v[2]);
        }) {
            ratatui::restore();
            crate::fatal(&format!("terminal error: {e}"));
        }

        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                match &mut app.input {
                    InputState::Creating { buffer, cursor } => match key.code {
                        KeyCode::Enter => {
                            let name = buffer.trim().to_string();
                            if !name.is_empty() && !app.config.profiles.contains_key(&name) {
                                app.config.profiles.insert(name.clone(), config::Profile::default());
                                config::save_config(&app.config);
                                app.names.clear();
                                app.names.extend(app.config.profiles.keys().cloned());
                                app.names.sort();
                                if let Some(pos) = app.names.iter().position(|n| n == &name) {
                                    app.profile_list.select(Some(pos));
                                }
                            }
                            app.input = InputState::None;
                        }
                        KeyCode::Esc => app.input = InputState::None,
                        KeyCode::Backspace => {
                            if *cursor > 0 {
                                buffer.remove(*cursor - 1);
                                *cursor -= 1;
                            }
                        }
                        KeyCode::Left => *cursor = cursor.saturating_sub(1),
                        KeyCode::Right => *cursor = (*cursor + 1).min(buffer.len()),
                        KeyCode::Char(c) => {
                            buffer.insert(*cursor, c);
                            *cursor += 1;
                        }
                        _ => {}
                    },
                    InputState::Editing { buffer, cursor } => match key.code {
                        KeyCode::Enter => {
                            let saved = buffer.clone();
                            let name = app.selected_name().map(|s| s.to_string());
                            if let Some(ref name) = name {
                                if let Some(profile) = app.config.profiles.get_mut(name) {
                                    let field = &PROFILE_FIELDS[app.field_idx];
                                    (field.set)(profile, saved);
                                    config::save_config(&app.config);
                                }
                            }
                            app.input = InputState::None;
                        }
                        KeyCode::Esc => {
                            app.input = InputState::None;
                        }
                        KeyCode::Backspace => {
                            if *cursor > 0 {
                                buffer.remove(*cursor - 1);
                                *cursor -= 1;
                            }
                        }
                        KeyCode::Left => {
                            *cursor = cursor.saturating_sub(1);
                        }
                        KeyCode::Right => {
                            *cursor = (*cursor + 1).min(buffer.len());
                        }
                        KeyCode::Char(c) => {
                            buffer.insert(*cursor, c);
                            *cursor += 1;
                        }
                        _ => {}
                    },
                    InputState::None => match app.focus {
                        Focus::Left => match key.code {
                            KeyCode::Enter => {
                                let name = app.selected_name().map(|s| s.to_string());
                                ratatui::restore();
                                return (app, name);
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                ratatui::restore();
                                return (app, None);
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                ratatui::restore();
                                return (app, None);
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.profile_next(),
                            KeyCode::Up | KeyCode::Char('k') => app.profile_prev(),
                            KeyCode::Right => app.focus = Focus::Right,
                            KeyCode::Char('n') => {
                                app.input = InputState::Creating {
                                    buffer: String::new(),
                                    cursor: 0,
                                };
                            }
                            _ => {}
                        },
                        Focus::Right => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                ratatui::restore();
                                return (app, None);
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.field_next(),
                            KeyCode::Up | KeyCode::Char('k') => app.field_prev(),
                            KeyCode::Left => app.focus = Focus::Left,
                            KeyCode::Enter => {
                                let name = app.selected_name().map(|s| s.to_string());
                                if let Some(ref name) = name {
                                    if let Some(profile) = app.config.profiles.get(name) {
                                        let field = &PROFILE_FIELDS[app.field_idx];
                                        let value = (field.get)(profile).unwrap_or_default();
                                        let len = value.len();
                                        app.input = InputState::Editing {
                                            buffer: value,
                                            cursor: len,
                                        };
                                    }
                                }
                            }
                            _ => {}
                        },
                    },
                }
            }
            Err(_) => {
                ratatui::restore();
                crate::fatal("failed to read input");
            }
            _ => {}
        }
    }
}
