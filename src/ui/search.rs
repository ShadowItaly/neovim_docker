use shiplift::{Docker,ContainerListOptions,rep,RmContainerOptions};
use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout,Alignment},
    style::{Color, Modifier, Style},
    text::{Span,Text,Spans},
    widgets::{Block, Borders, Paragraph,List, ListItem,ListState},
    Terminal,
};
use termion::event::Key;
use termion::input::TermRead;
use crate::ui::AppState;
use crate::ui::popup::AppPopupContext;
use std::process::Command;
use termion::raw::{IntoRawMode};
use termion::screen::{ToAlternateScreen,ToMainScreen};
use crate::VERSION;

struct ContainerList {
    containers: Vec<rep::Container>,
    filtered_list: Vec<usize>,
    selected_state: ListState,
}

impl ContainerList {
    pub async fn update(&mut self, docker: &Docker) {
        let val = docker.containers();
        let opts = ContainerListOptions::builder().all().build();
        self.containers = val.list(&opts).await.unwrap();
        self.containers.retain(|x| {
            for name in x.names.iter() {
                if name.starts_with("/") {
                    return true;
                }
            }
            false 
        });

        self.filtered_list = (0..self.containers.len()).collect();
        self.selected_state.select(Some(0));
    }

    pub fn select_next(&mut self) {
        if let Some(mut x) = self.selected_state.selected() {
            x+=1;
            if x > self.filtered_list.len() {
                x = 0;
            }
            self.selected_state.select(Some(x));
        }
    }

    pub fn select_last(&mut self) {
        if let Some(x) = self.selected_state.selected() {
            let mut res = x.checked_sub(1);
            if res.is_none() {
                res = Some(self.filtered_list.len());
            }
            self.selected_state.select(res);
        }
    }

    pub fn update_filtered_list(&mut self, search: &str) {
        self.filtered_list = self.containers.iter().enumerate().filter_map(|(idx,y)| if y.names[0].find(search).is_some() {
            Some(idx)
        } 
        else { 
            None 
        }).collect();
        self.selected_state.select(Some(0));
    }

    pub fn get_selected(&self) -> usize {
        if let Some(sel) = self.selected_state.selected() {
            if sel < self.filtered_list.len() {
                return sel;
            }
        }
        self.containers.len()
    }

    pub fn get_expanded_string(&self, idx: usize) -> String {
        if self.filtered_list.len() < 9 {
            return idx.to_string();
        }
        else if self.filtered_list.len() < 98 {
            if idx < 10 {
                return String::from("0")+&idx.to_string();
            }
            else {
                return idx.to_string();
            }
        }
        panic!("At the moment only up to 100 container supported!");
    }

    pub fn get_selected_by_string(&self, selection: &str) -> Vec<usize> {
        let mut results = Vec::new();
        for x in 0..self.filtered_list.len()+1 {
            if self.get_expanded_string(x).starts_with(selection) {
                results.push(x);
            }
        }
        results 
    }

    pub fn as_gui_element<'a>(&'a mut self, _search: &str, selection: &'a str) -> (Vec<ListItem>,&'a mut ListState) {
        let mut item_list = self.filtered_list.iter().enumerate().map(|(idx,x)| { 
            let result = self.get_expanded_string(idx);
            let status = if self.containers[*x].state == "running" {
                Span::styled("[RUNNING]",Style::default().fg(Color::Green))
            }
            else {
                Span::styled("[STOPPED]",Style::default().fg(Color::Red))
            };
            let index = if result.starts_with(selection) {
                vec![Span::styled(selection, Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),Span::raw(format!("{}. ",&result[selection.len()..])),Span::raw(self.containers[*x].names[0].clone()),Span::raw(" - "),status]
            }
            else {
                vec![Span::raw(format!("{}. ",result)),Span::raw(self.containers[*x].names[0].clone()),Span::raw(" - "),status]
            };
            ListItem::new(Spans::from(index))
            }).collect::<Vec<ListItem>>();

            let idx = self.get_expanded_string(self.filtered_list.len());
            let index = if idx.starts_with(selection) {
                vec![Span::styled(selection, Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),Span::raw(format!("{}. ",&idx[selection.len()..])),Span::raw("Create new container development environment")]
            }
            else {
                vec![Span::raw(format!("{}. ",idx)),Span::raw("Create new container development environment")]
            };

            item_list.push(ListItem::new(Spans::from(index)));
            (item_list,&mut self.selected_state)
    }
}

enum CurrentState {
    EnteringSearch,
    SelectingOption,
    CommandMode,
}

pub struct AppSearchContext {
    container_list: ContainerList,
    search_term: String,
    current_selection: String,
    current_state: CurrentState,
}

async fn attach_to_container(id: &str) {
    let args = vec!["attach",id,"--detach-keys","ctrl-d"];
    println!("{}",ToMainScreen);
    Command::new("docker").args(args).spawn().unwrap().wait().unwrap();
    println!("{}",ToAlternateScreen);
}

impl AppSearchContext{ 
    pub fn new() -> AppSearchContext { 
        AppSearchContext { 
            container_list: ContainerList {
                containers: Vec::new(),
                filtered_list: Vec::new(),
                selected_state: ListState::default(),
            },
            current_selection: String::new(),
            search_term: String::new(),
            current_state: CurrentState::CommandMode,
        }
    }

    pub async fn update(&mut self, docker: &Docker) {
        self.container_list.update(docker).await;
    }

    pub fn render<B: Backend>(&mut self,terminal: &mut Terminal<B>, popup: Option<&mut AppPopupContext>) {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1),Constraint::Length(3),Constraint::Min(2),Constraint::Length(3)].as_ref())
                .split(f.size());

            let mut style_non = Style::default().fg(Color::Blue);
            let mut style_hi = Style::default().fg(Color::LightGreen);
            let mut style_help = Style::default().fg(Color::Cyan);
            let mut header_style = Style::default().fg(Color::LightRed);
            let mut header_other_style = Style::default().bg(Color::Black).fg(Color::LightYellow);
            let mut list_highlight_style = Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD);


            if popup.is_some() {
                let style_dim = Style::default().fg(Color::Gray).add_modifier(Modifier::DIM);
                style_non = style_dim;
                style_hi = style_dim;
                style_help = style_dim;
                header_style = style_dim;
                header_other_style = style_dim;
                list_highlight_style = style_dim;
            }

            let header_paragraph = Paragraph::new(Spans::from(vec![Span::styled(VERSION,header_style),Span::raw(" | Search mode")]))
                .style(header_other_style)
                .alignment(Alignment::Center);
            let mut paragraph = Paragraph::new(Span::raw(&self.search_term))
                                    .block(Block::default().borders(Borders::ALL).title("Search"))
                                    .alignment(Alignment::Left);
            paragraph = match self.current_state {
                CurrentState::EnteringSearch => paragraph.style(style_hi),
                _ => paragraph.style(style_non),
            };

            f.render_widget(header_paragraph,chunks[0]);
            f.render_widget(paragraph,chunks[1]);

            let (result,state) = self.container_list.as_gui_element(&self.search_term,&self.current_selection);
            let mut block = Block::default().borders(Borders::ALL).title(format!("Selection: {}",&self.current_selection));
            block = match self.current_state {
                CurrentState::SelectingOption => block.style(style_hi),
                _ => block.style(style_non)
            };

            let container = List::new(result)
                .block(block)
                .style(style_non)
                .highlight_style(list_highlight_style).highlight_symbol(">> ");
            match self.current_state {
                CurrentState::SelectingOption => {
                    f.render_stateful_widget(container,chunks[2],state);
                },
                _ => {
                    f.render_widget(container, chunks[2]);
                }
            }

            let help = match self.current_state {
                CurrentState::EnteringSearch => {
                    Paragraph::new(Text::from("Enter name - accept with <enter>; quit with <ctrl-c>")).style(style_help)
                                    .block(Block::default().borders(Borders::ALL).title("Help"))
                                    .alignment(Alignment::Left)
                },
                CurrentState::SelectingOption => {
                    Paragraph::new(Text::from("Select container: accept - <enter>; back to name field - <esc>; quit - <ctrl-c>; w - up; s - down")).style(style_help)
                                    .block(Block::default().borders(Borders::ALL).title("Help"))
                                    .alignment(Alignment::Left)
                },
                CurrentState::CommandMode => {
                    Paragraph::new(Text::from("/ : Enter search mode, 0-9 : Select container, <space> : Select container")).style(style_help)
                                    .block(Block::default().borders(Borders::ALL).title("Help"))
                                    .alignment(Alignment::Left)
                }
            };
            f.render_widget(help,chunks[3]);
            if popup.is_some() {
                popup.unwrap().render_on(f);
            }
        }).unwrap();
        terminal.show_cursor().unwrap();
        terminal.set_cursor(self.search_term.len() as u16+1, 2).unwrap();
    }

    pub async fn event_loop<B: Backend>(&mut self, term: &mut Terminal<B>, docker: &Docker) -> AppState {
        self.update(docker).await;
        self.container_list.update_filtered_list(&self.search_term);
        self.current_selection.clear();

        let stdin = io::stdin();
        self.render(term,None);
        for evt in stdin.keys() {
            match evt {
                Ok(Key::Char(r)) => {
                    match self.current_state {
                        CurrentState::EnteringSearch => {
                            if r == '\n' {
                                self.current_state = CurrentState::CommandMode;
                            }
                            else {
                                self.search_term.push(r);
                                self.container_list.update_filtered_list(&self.search_term);
                            }
                        },
                        CurrentState::CommandMode => {
                            if r == '/' {
                                self.current_state = CurrentState::EnteringSearch;
                            }
                            else if r == 'q' {
                                return AppState::Exiting;
                            }
                            else if r >= '0' && r <= '9' {
                                self.current_selection.push(r);
                                let num_results = self.container_list.get_selected_by_string(&self.current_selection);
                                if num_results.len() == 0 {
                                    self.current_selection.clear();
                                }
                                else if num_results.len() == 1 {
                                    let mut selected = num_results[0];
                                    if selected == self.container_list.filtered_list.len() {
                                        return AppState::NewContainer;
                                    }
                                    else {
                                        selected = self.container_list.filtered_list[selected];
                                        let cont = self.container_list.containers.iter().filter_map(|x| if &x.names[0] == &self.container_list.containers[selected].names[0] { Some(&x.id) } else { None }).collect::<Vec<&String>>();
                                        let container = docker.containers().get(cont[0]);
                                        if let Ok(inspection) = container.inspect().await {
                                            if inspection.state.running {
                                                attach_to_container(cont[0]).await;
                                                term.clear().unwrap();
                                            }
                                            else {
                                                //Start it
                                                container.start().await.unwrap();
                                                attach_to_container(cont[0]).await;
                                                term.clear().unwrap();
                                            }
                                            self.container_list.update(&docker).await;
                                            self.search_term.clear();
                                            self.current_selection.clear();
                                            self.container_list.update_filtered_list(&self.search_term);
                                        }
                                        else {
                                            AppPopupContext::new("An error occured press any key to continue".to_owned()).style(Style::default().fg(Color::LightRed)).event_render_loop(|popup| self.render(term, Some(popup)));
                                        }
                                    }
                                }
                            }
                            else if r == ' ' {
                                self.current_state = CurrentState::SelectingOption;
                            }
                        }
                        CurrentState::SelectingOption => {
                            if r == 'w' {
                                self.container_list.select_last();
                            }
                            else if r == 's' {
                                self.container_list.select_next();
                            }
                            else if r == 'd' {
                                let selected = self.container_list.get_selected();
                                if selected != self.container_list.containers.len() {
                                    let result = AppPopupContext::decision("Do you really want to delete this container (y) - yes, (n) - no?".to_owned()).style(Style::default().fg(Color::LightRed)).event_render_loop(|popup| self.render(term, Some(popup)));
                                    if result == "yes" {
                                        let cont = &self.container_list.containers[selected].id;
                                        let _ = docker.containers().get(cont).stop(None).await;
                                        let opts = RmContainerOptions::builder().build();
                                        let res = docker.containers().get(cont).remove(opts).await;
                                        if res.is_err() {
                                            AppPopupContext::new("Could not remove container.".to_owned()).style(Style::default().fg(Color::LightRed)).event_render_loop(|popup| self.render(term, Some(popup)));
                                        }
                                        self.update(docker).await;
                                        self.container_list.update_filtered_list(&self.search_term);
                                    }
                                }
                            }
                            else if r == 'e' {
                                let selected = self.container_list.get_selected();
                                if selected != self.container_list.containers.len() {
                                    let cont = &self.container_list.containers[selected].id;
                                    let res = docker.containers().get(cont).stop(None).await;
                                    if res.is_err() {
                                        AppPopupContext::new("Could not stop container.".to_owned()).style(Style::default().fg(Color::LightRed)).event_render_loop(|popup| self.render(term, Some(popup)));
                                    }
                                    self.update(docker).await;
                                    self.container_list.update_filtered_list(&self.search_term);
                                }
                            }
                            else if r == '\n' || r == '\t' {
                                let mut selected = self.container_list.get_selected();
                                if selected == self.container_list.filtered_list.len() {
                                    return AppState::NewContainer;
                                }
                                else {
                                    selected = self.container_list.filtered_list[selected];
                                    let cont = self.container_list.containers.iter().filter_map(|x| if &x.names[0] == &self.container_list.containers[selected].names[0] { Some(&x.id) } else { None }).collect::<Vec<&String>>();
                                    let container = docker.containers().get(cont[0]);
                                    if let Ok(inspection) = container.inspect().await {
                                        if inspection.state.running {
                                            attach_to_container(cont[0]).await;
                                            term.clear().unwrap();
                                        }
                                        else {
                                            //Start it
                                            container.start().await.unwrap();
                                            attach_to_container(cont[0]).await;
                                            term.clear().unwrap();
                                        }
                                        self.container_list.update(&docker).await;
                                        self.search_term.clear();
                                        self.current_selection.clear();
                                        self.container_list.update_filtered_list(&self.search_term);
                                    }
                                    else {
                                        AppPopupContext::new("An error occured press any key to continue".to_owned()).style(Style::default().fg(Color::LightRed)).event_render_loop(|popup| self.render(term, Some(popup)));
                                    }
                                }
                            }
                        },
                    }
                },
                Ok(Key::Backspace) => {
                    match self.current_state {
                        CurrentState::EnteringSearch => {
                            self.search_term.pop();
                            self.container_list.update_filtered_list(&self.search_term);
                        },
                        _ => {}
                    }
                },
                Ok(Key::Esc) => {
                    self.current_state = CurrentState::CommandMode;
                    self.current_selection.clear();
                },
                Ok(Key::Ctrl('c')) => {
                    return AppState::Exiting;
                },
                Ok(Key::Ctrl('h')) => {
                    return AppState::Help;
                },
                Ok(Key::Ctrl('a')) => {
                    return AppState::Command;
                },
                _ => {}
            }
            self.render(term,None);
        }
        return AppState::Exiting;
    }
}
