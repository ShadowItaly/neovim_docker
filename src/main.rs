use std::io;
use shiplift::Docker;
use termion::{raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    Terminal,
};
use termion::color;

mod ui;

const VERSION: &'static str = concat!("Docker development environment version v",env!("CARGO_PKG_VERSION"));


#[tokio::main]
async fn main() {
    let docker = Docker::new();
    println!("{}{}{}",color::Fg(color::LightMagenta),VERSION,color::Fg(color::Reset));
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    let mut app = ui::App::new(docker).await;
    app.event_loop(&mut terminal).await;
}
