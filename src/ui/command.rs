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

pub struct AppCommandContext {
    command_term: String,
    output_log: String,
}

impl AppCommandContext { 
    pub fn new() -> AppCommandContext { 
        AppCommandContext { 
            command_term: String::new(),
            output_log: String::new(),
        }
    }

    pub fn render<B: Backend>(&mut self,terminal: &mut Terminal<B>) {
        //FIX: allright :D
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1),Constraint::Length(3),Constraint::Min(1),Constraint::Length(3)].as_ref())
                .split(f.size());

            let style_non = Style::default().fg(Color::Blue);
            let style_hi = Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD);

            let header_paragraph = Paragraph::new(Spans::from(vec![Span::styled(VERSION,Style::default().fg(Color::LightRed)),Span::raw(" | Command mode")]))
                .style(Style::default().bg(Color::Black).fg(Color::LightYellow))
                .alignment(Alignment::Center);
            let paragraph = Paragraph::new(Span::raw(format!(">> {}",&self.command_term))).style(style_hi.clone())
                                    .block(Block::default().borders(Borders::ALL).title("Command line"))
                                    .alignment(Alignment::Left);
            let style_help = Style::default().fg(Color::Cyan);
            f.render_widget(header_paragraph,chunks[0]);
            f.render_widget(paragraph,chunks[1]);
            let log = Paragraph::new(Text::from(&self.output_log[..])).style(style_non)
                                    .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Gray)).title("Command output"))
                                    .alignment(Alignment::Left);
            let help = Paragraph::new(Text::from("Ctrl+h - Show full help")).style(style_help)
                                    .block(Block::default().borders(Borders::ALL).title("Help"))
                                    .alignment(Alignment::Left);
            f.render_widget(log,chunks[2]);
            f.render_widget(help,chunks[3]);
        }).unwrap();
        terminal.show_cursor().unwrap();
        terminal.set_cursor(self.command_term.len() as u16+4, 2).unwrap();
    }

    pub fn event_loop<B: Backend>(&mut self, term: &mut Terminal<B>) -> AppState {
        let stdin = io::stdin();
        self.render(term);
        for evt in stdin.keys() {
            match evt {
                Ok(Key::Char(r)) => {
                    if r == '\n' {
                        if self.command_term == "quit" {
                            return AppState::Exiting;
                        }
                        else {
                            self.output_log = format!("Unkown command \"{}\"\n",self.command_term);
                            self.command_term.clear();
                        }
                    }
                    else {
                        self.command_term.push(r);
                    }
                },
                Ok(Key::Backspace) => {
                    self.command_term.pop();
                },
                Ok(Key::Ctrl('c')) => {
                    return AppState::Exiting;
                }
                Ok(Key::Ctrl('h')) => {
                    return AppState::Help;
                },
                Ok(Key::Ctrl('s')) => {
                    return AppState::Search;
                },
                _ => {}
            }
            self.render(term);
        }
        return AppState::Exiting;
    }
}
