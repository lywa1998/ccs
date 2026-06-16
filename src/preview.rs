use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use crate::config::{self, Config};
use crate::{ACCENT, MUTED};

pub fn build_preview(config: &Config, name: &str) -> Text<'static> {
    let Some(profile) = config.profiles.get(name) else {
        return Text::default();
    };

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref desc) = profile.description {
        lines.push(Line::from(Span::styled(
            desc.clone(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));
    }

    if let Ok(resolved) = config::resolve_profile(config, name) {
        let env_map = config::build_env(&resolved, false);
        let has_provider = resolved.provider.is_some();

        // Model section
        let model_keys: &[&str] = &[
            "ANTHROPIC_MODEL",
            "ANTHROPIC_SMALL_FAST_MODEL",
            "ANTHROPIC_DEFAULT_HAIKU_MODEL",
            "ANTHROPIC_DEFAULT_SONNET_MODEL",
            "ANTHROPIC_DEFAULT_OPUS_MODEL",
        ];
        let mut shown_models = false;
        for key in model_keys {
            if let Some(val) = env_map.get(*key) {
                if !shown_models {
                    lines.push(Line::from(Span::styled(
                        "Models",
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    )));
                    shown_models = true;
                }
                let mut line = Line::default();
                line.push_span(Span::styled(
                    format!("  {:<36}", key),
                    Style::default().fg(MUTED),
                ));
                line.push_span(Span::styled(val.clone(), Color::Green));
                lines.push(line);
            }
        }

        // Provider section
        if has_provider {
            if shown_models {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "Provider",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )));
            for key in &["ANTHROPIC_BASE_URL", "ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"] {
                if let Some(val) = env_map.get(*key) {
                    let mut line = Line::default();
                    line.push_span(Span::styled(
                        format!("  {:<36}", key),
                        Style::default().fg(MUTED),
                    ));
                    let val_style = if *key == "ANTHROPIC_API_KEY" && val == "(cleared)" {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    };
                    line.push_span(Span::styled(val.clone(), val_style));
                    lines.push(line);
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                " $ claude ",
                Style::default()
                    .fg(Color::Black)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" --dangerously-skip-permissions"),
        ]));
    }

    Text::from(lines)
}
