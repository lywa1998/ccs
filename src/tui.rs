use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::config::Config;
use crate::preview;
use crate::{ACCENT, MUTED};

pub struct App {
    config: Config,
    names: Vec<String>,
    list_state: ListState,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut names: Vec<String> = config.profiles.keys().cloned().collect();
        names.sort();
        let mut list_state = ListState::default();
        if !names.is_empty() {
            list_state.select(Some(0));
        }
        Self { config, names, list_state }
    }

    fn selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    fn selected_name(&self) -> Option<&str> {
        self.names.get(self.selected_index()).map(|s| s.as_str())
    }

    fn next(&mut self) {
        if self.names.is_empty() {
            return;
        }
        let i = (self.selected_index() + 1) % self.names.len();
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.names.is_empty() {
            return;
        }
        let i = if self.selected_index() == 0 {
            self.names.len() - 1
        } else {
            self.selected_index() - 1
        };
        self.list_state.select(Some(i));
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

pub fn run(mut app: App) -> (App, Option<String>) {
    let mut terminal = ratatui::init();

    loop {
        if let Err(e) = terminal.draw(|frame| {
            let area = frame.area();

            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // header
                    Constraint::Min(0),     // body
                    Constraint::Length(1),  // footer
                ])
                .split(area);

            // ── header ──
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    "cc-switch",
                    Style::default()
                        .fg(ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  profile manager for Claude Code",
                    Style::default().fg(MUTED),
                ),
            ]));
            frame.render_widget(
                header.block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(MUTED)),
                ),
                v[0],
            );

            // ── body: list + preview ──
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .split(v[1]);

            // ── left: profile list ──
            let items: Vec<ListItem> = app
                .names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let desc = app.config.profiles[name]
                        .description
                        .as_deref()
                        .unwrap_or("");
                    let is_selected = Some(i) == app.list_state.selected();
                    let name_style = if is_selected {
                        Style::default()
                            .fg(ACCENT)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    if desc.is_empty() {
                        ListItem::new(Line::from(Span::styled(name.as_str(), name_style)))
                    } else {
                        ListItem::new(vec![
                            Line::from(Span::styled(name.as_str(), name_style)),
                            Line::from(Span::styled(desc, Style::default().fg(MUTED))),
                        ])
                    }
                })
                .collect();

            let count = app.names.len();
            let list = List::new(items)
                .block(
                    Block::bordered()
                        .border_style(Style::default().fg(ACCENT))
                        .title(Span::styled(
                            " Profiles ",
                            Style::default()
                                .fg(ACCENT)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .title_bottom(Span::styled(
                            format!(" {count} profiles "),
                            Style::default().fg(MUTED),
                        )),
                )
                .highlight_style(Style::default().fg(Color::Black).bg(ACCENT))
                .highlight_symbol(" ▸");

            frame.render_stateful_widget(list, body[0], &mut app.list_state);

            // ── right: preview ──
            let selected = app.selected_name().unwrap_or("");
            let preview_content = preview::build_preview(&app.config, selected);

            let preview_block = Block::bordered()
                .border_style(Style::default().fg(ACCENT))
                .title(Span::styled(
                    format!(" {} ", selected),
                    Style::default()
                        .fg(ACCENT)
                        .add_modifier(Modifier::BOLD),
                ));

            frame.render_widget(
                Paragraph::new(preview_content)
                    .block(preview_block)
                    .wrap(Wrap { trim: false }),
                body[1],
            );

            // ── footer ──
            let footer = Line::from(vec![
                footer_span("↑↓", "navigate"),
                Span::raw(" "),
                footer_span("Enter", "select"),
                Span::raw(" "),
                footer_span("q/ESC", "quit"),
            ]);
            frame.render_widget(Paragraph::new(footer).centered(), v[2]);
        }) {
            ratatui::restore();
            crate::fatal(&format!("terminal error: {e}"));
        }

        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
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
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                _ => {}
            },
            Err(_) => {
                ratatui::restore();
                crate::fatal("failed to read input");
            }
            _ => {}
        }
    }
}
