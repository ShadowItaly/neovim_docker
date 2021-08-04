use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout,Rect,Alignment},
    style::Style,
    text::Span,
    widgets::{Block, Borders, Paragraph,Clear,Wrap,BorderType},
    Terminal,
    terminal::Frame,
};
use termion::input::TermRead;
use termion::event::Key;
 

enum PopupTask {
    YesNoDecision,
    Information,
}

pub struct AppPopupContext {
    message: String,
    style: Style,
    task: PopupTask,
}

impl AppPopupContext { 
    pub fn new(message: String) -> AppPopupContext {
        AppPopupContext {
            message,
            style: Style::default(),
            task: PopupTask::Information,
        }
    }

    pub fn decision(message: String) -> AppPopupContext {
        AppPopupContext {
            message,
            style: Style::default(),
            task: PopupTask::YesNoDecision,
        }
    }

    pub fn style(mut self, style: Style) -> AppPopupContext {
        self.style = style;
        self
    }

    /// helper function to create a centered rect using up
    /// certain percentage of the available rect `r`
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Length(3),
                Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    pub fn render<B: Backend>(&mut self,terminal: &mut Terminal<B>) {
        terminal.draw(|mut f| {
            self.render_on(&mut f);
        }).unwrap();
    }

    pub fn render_on<B: Backend>(&mut self, f: &mut Frame<B>) {
        let popup_layout = AppPopupContext::centered_rect(80, 20, f.size());
        let block = Paragraph::new(Span::raw(&self.message)).block(Block::default().title(" Popup message ").borders(Borders::ALL).style(self.style).border_type(BorderType::Double)).style(self.style).wrap(Wrap { trim: true }).alignment(Alignment::Center);
        let bigger_rect = Rect::new(popup_layout.x-1,popup_layout.y-1,popup_layout.width+2,popup_layout.height+2);
        f.render_widget(Clear, bigger_rect);
        f.render_widget(block, popup_layout);
    }

    pub fn event_render_loop<F: FnMut(&mut AppPopupContext)>(&mut self, mut render_func: F) -> String {
        let stdin = io::stdin();
        render_func(self);
        for evt in stdin.keys() {
            match self.task {
                PopupTask::Information => {
                    match evt {
                        Ok(_) => {
                            return String::new();
                        },
                        _ => {}
                    }
                },
                PopupTask::YesNoDecision => {
                    match evt {
                        Ok(Key::Char(r)) => {
                            if r == 'y' || r == 'Y' {
                                return String::from("yes");
                            }
                            else if r == 'n' || r == 'N' {
                                return String::from("no");
                            }
                        },
                        _ => {}
                    }
                },
            }
            render_func(self);
        }
        String::new()
    }

    pub fn event_loop<B: Backend>(&mut self, term: &mut Terminal<B>) -> String {
        self.event_render_loop(|app| app.render(term))
    }
}
