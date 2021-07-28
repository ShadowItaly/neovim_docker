use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout,Alignment},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph,Clear,Wrap},
    Terminal,
};
use termion::input::TermRead;
use crate::ui::AppState;


pub struct AppErrorContext {
    error: String,
}

impl AppErrorContext {
    pub fn new() -> AppErrorContext {
        AppErrorContext {
            error: String::new(),
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.error = error;
    }

    pub fn render<B: Backend>(&mut self,terminal: &mut Terminal<B>) {
        terminal.draw(|f| {
            let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                Constraint::Min(3),
                Constraint::Percentage(90),
                ]
                .as_ref(),
            )
            .split(f.size());
            let block = Paragraph::new(Span::raw(&self.error)).block(Block::default().title("Critical error").borders(Borders::ALL).style(Style::default().fg(Color::White))).style(Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)).wrap(Wrap { trim: true }).alignment(Alignment::Center);
            f.render_widget(Clear, popup_layout[0]);
            f.render_widget(block, popup_layout[0]);
        }).unwrap();
    }

    pub fn event_loop<B: Backend>(&mut self, term: &mut Terminal<B>) -> AppState {
        let stdin = io::stdin();
        self.render(term);
        for evt in stdin.keys() {
            match evt {
                Ok(_) => {
                    return AppState::Exiting;
                },
                _ => {}
            }
            self.render(term);
        }
        AppState::Exiting
    }
}
