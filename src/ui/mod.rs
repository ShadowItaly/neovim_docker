use shiplift::{Docker};
use tui::{
    backend::Backend,
    Terminal,
};

mod search;
mod error;
mod command;
mod help;
mod popup;
mod new_container;



pub enum AppState {
    Search,
    Error(String),
    Command,
    Exiting,
    Help,
    NewContainer,
}

pub struct App {
    docker: Docker,
    state: AppState,
    error_context: error::AppErrorContext,
    search_context: search::AppSearchContext,
    command_context: command::AppCommandContext,
    help_context: help::AppHelpContext,
}

impl App {
    pub async fn new(docker: Docker) -> App {
        let mut search_context = search::AppSearchContext::new();
        let state = match docker.info().await {
            Ok(_) => {
                search_context.update(&docker).await;
                AppState::Search
            },
            Err(_) => AppState::Error("Could not connect to docker! Is docker installed? Press any key to quit.".to_owned()),
        };


        App {
            docker,
            state,
            error_context: error::AppErrorContext::new(),
            search_context: search_context,
            command_context: command::AppCommandContext::new(),
            help_context: help::AppHelpContext::new(),
        }
    }

    pub async fn event_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) {
        loop {
            self.state = match &self.state {
                AppState::Error(error) => {
                    self.error_context.set_error(error.clone());
                    self.error_context.event_loop(terminal)
                },
                AppState::Search => {
                    self.search_context.event_loop(terminal,&self.docker).await
                },
                AppState::Command => {
                    self.command_context.event_loop(terminal)
                },
                AppState::Help => {
                    self.help_context.event_loop(terminal)
                },
                AppState::NewContainer => {
                    new_container::AppNewContainerContext::new().event_loop(terminal,&self.docker).await
                },
                AppState::Exiting => {
                    return;
                }
            };
        }
    }
}
