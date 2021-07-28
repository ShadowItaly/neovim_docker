use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout,Alignment},
    style::{Color, Modifier, Style},
    text::{Span,Text,Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use termion::event::Key;
use termion::input::TermRead;
use crate::ui::AppState;
use crate::VERSION;

pub struct AppHelpContext {
}

impl AppHelpContext { 
    pub fn new() -> AppHelpContext { 
        AppHelpContext { 
        }
    }

    pub fn render<B: Backend>(&mut self,terminal: &mut Terminal<B>) {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1),Constraint::Min(1)].as_ref())
                .split(f.size());

            let style_hi = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);

            let header_paragraph = Paragraph::new(Spans::from(vec![Span::styled(VERSION,Style::default().fg(Color::LightRed)),Span::raw(" | Help window")]))
                .style(Style::default().bg(Color::Black).fg(Color::LightYellow))
                .alignment(Alignment::Center);
            let paragraph = Paragraph::new(Text::from(format!("Ctrl+h - Show help\nCtrl-a - Command mode\nCtrl-s - Search mode\nCtrl-c - quit programm\nPage up - Scroll results up\nPage down - Scroll results down\n\nCommand mode commands:\n-- help - Show commands"))).style(style_hi)
                                    .block(Block::default().borders(Borders::ALL).title("Help"))
                                    .alignment(Alignment::Left);
            f.render_widget(header_paragraph,chunks[0]);
            f.render_widget(paragraph,chunks[1]);
        }).unwrap();
    }

    pub fn event_loop<B: Backend>(&mut self, term: &mut Terminal<B>) -> AppState {
        let stdin = io::stdin();
        self.render(term);
        for evt in stdin.keys() {
            match evt {
                Ok(Key::Ctrl('c')) => {return AppState::Exiting;},
                Ok(Key::Ctrl('a')) => {return AppState::Command;},
                Ok(Key::Ctrl('s')) => {return AppState::Search;},
                _ => {}
            }
            self.render(term);
        }
        return AppState::Exiting;
    }
}
