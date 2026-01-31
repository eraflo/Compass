// Copyright 2026 eraflo
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::core::analysis::recovery::RecoveryRecommendation;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, area: Rect, recommendation: &RecoveryRecommendation) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ])
        .split(area);

    let rect = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(popup_layout[1])[1];

    frame.render_widget(Clear, rect);

    let block = Block::default()
        .title(" 💡 Smart Recovery ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let mut text = vec![
        Line::from(vec![
            Span::styled("Analysis: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&recommendation.message),
        ]),
        Line::from(""),
    ];

    if let Some(cmd) = &recommendation.fix_command {
        text.push(Line::from(Span::styled(
            "Suggested Fix:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )));
        text.push(Line::from(Span::styled(
            format!("$ {}", cmd),
            Style::default().bg(Color::Black).fg(Color::White),
        )));
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::raw("Press "),
            Span::styled("ENTER", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to apply this fix, or "),
            Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to ignore."),
        ]));
    } else {
        text.push(Line::from(vec![
            Span::raw("Press "),
            Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to close."),
        ]));
    }

    let p = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    frame.render_widget(p, rect);
}
