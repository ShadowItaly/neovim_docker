use shiplift::{Docker,ContainerOptions};
use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout,Alignment},
    style::{Color,Style,Modifier},
    text::{Span,Text,Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use termion::event::Key;
use termion::input::TermRead;
use crate::ui::AppState;
use crate::VERSION;
use tar::Builder;
use home;
use crate::ui::popup;

enum CurrentPhase {
    SelectName,
    SelectImage,
    EntryCommand,
    AddSSHKeys,
    GitConfig,
    AutoRemove,
    WorkingDirectory,
}

enum WorkingDirectorySetup {
    MountDirectory(String),
    CopyDirectory(String),
    DontUse,
}

pub struct AppNewContainerContext {
    container_name: String,
    auto_remove: String,
    entry_command: String,
    import_keys: String,
    git_config: String,
    image_name: String,
    working_dir: WorkingDirectorySetup,
    phase: CurrentPhase,
}

impl AppNewContainerContext { 
    pub fn new() -> AppNewContainerContext { 
        AppNewContainerContext { 
            container_name: String::new(),
            auto_remove: String::from("no"),
            image_name: String::from("shadowitaly/neovim_arch"),
            import_keys: String::from("yes"),
            git_config: String::from("yes"),
            entry_command: String::from("/bin/zsh"),
            working_dir: WorkingDirectorySetup::MountDirectory(std::env::current_dir().unwrap().to_str().unwrap().to_string()),
            phase: CurrentPhase::SelectName,
        }
    }

    pub fn render<B: Backend>(&mut self,terminal: &mut Terminal<B>,popup: Option<&mut popup::AppPopupContext>) {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3),Constraint::Length(3)].as_ref())
                .split(f.size());

            let mut style_non = Style::default().fg(Color::Blue);
            let mut style_hi = Style::default().fg(Color::LightGreen);
            let mut header_style = Style::default().fg(Color::LightRed);
            let mut header_other_style = Style::default().bg(Color::Black).fg(Color::LightYellow);
            let style_help = Style::default().fg(Color::Cyan);

            if popup.is_some() {
                let dim_style = Style::default().fg(Color::Gray).add_modifier(Modifier::DIM);
                style_non = dim_style;
                style_hi = dim_style;
                header_style = dim_style;
                header_other_style = dim_style;
            }

            let header_paragraph = Paragraph::new(Spans::from(vec![Span::styled(VERSION,header_style),Span::raw(" | New container creation")]))
                .style(header_other_style)
                .alignment(Alignment::Center);


            let mut paragraph = Paragraph::new(Span::raw(format!(">> {}",&self.container_name)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title("Enter new container name"));
            paragraph = paragraph.style(match self.phase {
                CurrentPhase::SelectName => {style_hi.clone()},
                _ => style_non.clone()
            });


            let mut image_name = Paragraph::new(Span::raw(format!(">> {}",&self.image_name)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title("Enter the image name"));
 
            image_name = image_name.style(match self.phase {
                CurrentPhase::SelectImage => style_hi.clone(),
                _ => style_non.clone()
            });
            f.render_widget(header_paragraph,chunks[0]);
            f.render_widget(paragraph,chunks[1]);
            f.render_widget(image_name,chunks[6]);

            let mut remove = Paragraph::new(Span::raw(format!(">> {}",&self.auto_remove)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title(" Autoremove (yes/no) "));
            remove = remove.style(match self.phase {
                CurrentPhase::AutoRemove => {style_hi.clone()},
                _ => style_non.clone()
            });
            f.render_widget(remove,chunks[2]);
    
            let mut entry_command = Paragraph::new(Span::raw(format!(">> {}",&self.entry_command)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title(" Entry command (Experienced users only!) "));
            entry_command = entry_command.style(match self.phase {
                CurrentPhase::EntryCommand => {style_hi.clone()},
                _ => style_non.clone()
            });
            f.render_widget(entry_command,chunks[3]);

            let mut import_ssh_key = Paragraph::new(Span::raw(format!(">> {}",&self.import_keys)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title(" Import SSH Keys (yes/no) "));
            import_ssh_key = import_ssh_key.style(match self.phase {
                CurrentPhase::AddSSHKeys => {style_hi.clone()},
                _ => style_non.clone()
            });
            f.render_widget(import_ssh_key,chunks[4]);

            let mut use_git_config = Paragraph::new(Span::raw(format!(">> {}",&self.git_config)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title(" Import Host git config (yes/no) "));
            use_git_config = use_git_config .style(match self.phase {
                CurrentPhase::GitConfig => {style_hi.clone()},
                _ => style_non.clone()
            });
            f.render_widget(use_git_config,chunks[5]);

            let mut path = match &self.working_dir {
                WorkingDirectorySetup::MountDirectory(x) => {
                    Paragraph::new(Span::raw(format!(">> {}",&x)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title("Mount this path inside the container: "))
                },
                WorkingDirectorySetup::CopyDirectory(x) => {
                    Paragraph::new(Span::raw(format!(">> {}",&x)))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title("Copy this path inside the container: "))
                },
                _ => {
                    Paragraph::new(Span::raw(">> --/--"))
                                    .alignment(Alignment::Left)
                                    .block(Block::default().borders(Borders::ALL).title("Neither mount nor copy working dir"))
                }
            };

            path = path.style(match self.phase {
                CurrentPhase::WorkingDirectory => {style_hi.clone()},
                _ => style_non.clone()
            });
            f.render_widget(path,chunks[7]);



            let help = Paragraph::new(Text::from("Ctrl+h - Show full help")).style(style_help)
                                    .block(Block::default().borders(Borders::ALL).title("Help"))
                                    .alignment(Alignment::Left);
            f.render_widget(help,chunks[8]);

            if popup.is_some() {
                popup.unwrap().render_on(f);
            }
        }).unwrap();
        match self.phase {
            CurrentPhase::SelectName => {
                terminal.show_cursor().unwrap();
                terminal.set_cursor(self.container_name.len() as u16+4, 2).unwrap();
            },
            CurrentPhase::AutoRemove => {
                terminal.show_cursor().unwrap();
                terminal.set_cursor(self.auto_remove.len() as u16+4,5).unwrap();
            },
            CurrentPhase::EntryCommand => {
                terminal.show_cursor().unwrap();
                terminal.set_cursor(self.entry_command.len() as u16+4,8).unwrap();
            },
            CurrentPhase::AddSSHKeys => {
                terminal.show_cursor().unwrap();
                terminal.set_cursor(self.import_keys.len() as u16+4,11).unwrap();
            },
            CurrentPhase::GitConfig => {
                terminal.show_cursor().unwrap();
                terminal.set_cursor(self.git_config.len() as u16+4,14).unwrap();
            },
            CurrentPhase::SelectImage => {
                terminal.show_cursor().unwrap();
                terminal.set_cursor(self.image_name.len() as u16+4,17).unwrap();
            },
            CurrentPhase::WorkingDirectory => {
                terminal.show_cursor().unwrap();
                let len = match &self.working_dir {WorkingDirectorySetup::MountDirectory(x)=>x.len(),WorkingDirectorySetup::CopyDirectory(x)=>x.len(),_ => 5};
                terminal.set_cursor(len as u16+4,20).unwrap();
            },
        }
    }

    pub async fn event_loop<B: Backend>(&mut self, term: &mut Terminal<B>,docker: &Docker) -> AppState {
        let stdin = io::stdin();

        self.render(term,None);
        for evt in stdin.keys() {
            match evt {
                Ok(Key::Char(r)) => {
                    match self.phase {
                        CurrentPhase::SelectName => {
                            if r == '\n' || r == '\t' {
                                if self.container_name == "" {
                                    popup::AppPopupContext::new("The controller name cannot be empty!".to_owned()).style(Style::default().fg(Color::LightRed)).event_render_loop(|popup| self.render(term,Some(popup)));
                                }
                                else {
                                    self.phase = CurrentPhase::AutoRemove;
                                }
                            }
                            else {
                                self.container_name.push(r);
                            }
                        },
                        CurrentPhase::AutoRemove => {
                            if r == '\n' || r == '\t'{
                                self.phase = CurrentPhase::EntryCommand;
                            }
                            else {
                                self.auto_remove.push(r);
                            }
                        },
                        CurrentPhase::EntryCommand => {
                            if r == '\n' || r == '\t'{
                                self.phase = CurrentPhase::AddSSHKeys;
                            }
                            else {
                                self.entry_command.push(r);
                            }
                        },
                        CurrentPhase::AddSSHKeys => {
                            if r == '\n' || r == '\t'{
                                self.phase = CurrentPhase::GitConfig;
                            }
                            else {
                                self.import_keys.push(r);
                            }
                        },
                        CurrentPhase::GitConfig => {
                            if r == '\n' || r == '\t'{
                                self.phase = CurrentPhase::SelectImage;
                            }
                            else {
                                self.git_config.push(r);
                            }
                        },
                        CurrentPhase::SelectImage => {
                            if r == '\n' || r == '\t'{
                                self.phase = CurrentPhase::WorkingDirectory;
                            }
                            else {
                                self.image_name.push(r);
                            }
                        },
                        CurrentPhase::WorkingDirectory => {
                            if r == '\n' {
                                let timezone = format!("TZ={}",std::fs::read_to_string("/etc/timezone").unwrap().trim());
                                let opts = ContainerOptions::builder(&self.image_name).auto_remove(self.auto_remove != "no").name(&(String::from("dde_")+&self.container_name)).cmd(self.entry_command.split(" ").collect()).tty(true).env(vec![&timezone]).attach_stdin(true).attach_stderr(true).attach_stdout(true).build();
                                let info = docker.containers().create(&opts).await.unwrap();
                                let container = info.id;
                                if self.import_keys == "yes" {
                                    let homed = home::home_dir().unwrap();
                                    let buffer : Vec<u8> = Vec::new();
                                    let mut ach = Builder::new(buffer);
                                    ach.append_dir_all(".ssh",homed.join(".ssh")).unwrap();
                                    let res = ach.into_inner().unwrap();
                                    docker.containers().get(&container).copy_to(std::path::Path::new("/root/"), res.into()).await.unwrap();
                                }
                                
                                if self.git_config == "yes" {
                                    let homed = home::home_dir().unwrap();
                                    let gitconfig = if homed.join(".gitconfig").exists() {
                                        std::fs::read_to_string(homed.join(".gitconfig")).unwrap()
                                    }
                                    else if std::path::Path::new("/etc/gitconfig").exists() {
                                        std::fs::read_to_string("/etc/gitconfig").unwrap()
                                    }
                                    else {
                                        String::from("")
                                    };
                                    if gitconfig != "" {
                                        docker.containers().get(&container).copy_file_into(std::path::Path::new("/root/.gitconfig"), gitconfig.as_bytes()).await.unwrap();
                                    }
                                    else {
                                        popup::AppPopupContext::new("Could not import host git config into container!".to_owned()).event_loop(term);
                                    }
                                }

                                return AppState::Search;
                            }
                            if r == '\t' {
                                match &self.working_dir {
                                    WorkingDirectorySetup::MountDirectory(x) => {self.working_dir = WorkingDirectorySetup::CopyDirectory(x.clone());}
                                    WorkingDirectorySetup::CopyDirectory(_) => {self.working_dir = WorkingDirectorySetup::DontUse;}
                                    _ => {self.working_dir = WorkingDirectorySetup::MountDirectory(std::env::current_dir().unwrap().to_str().unwrap().to_string());}
                                }
                            }
                        }
                    }
                },
                Ok(Key::Backspace) => {
                    match self.phase {
                        CurrentPhase::SelectName=> {
                            self.container_name.pop();
                        },
                        CurrentPhase::AutoRemove => {
                            self.auto_remove.pop();
                        },
                        CurrentPhase::EntryCommand => {
                            self.entry_command.pop();
                        },
                        CurrentPhase::AddSSHKeys => {
                            self.import_keys.pop();
                        },
                        CurrentPhase::GitConfig => {
                            self.git_config.pop();
                        },
                        CurrentPhase::SelectImage => {
                            self.image_name.pop();
                        },
                        CurrentPhase::WorkingDirectory => {
                            match &mut self.working_dir {
                                WorkingDirectorySetup::MountDirectory(x) => {x.pop();},
                                WorkingDirectorySetup::CopyDirectory(x) => {x.pop();},
                                _ => {}    
                            };
                        }
                    }
                },
                Ok(Key::Ctrl('c')) => {
                    return AppState::Exiting;
                }
                Ok(Key::Ctrl('h')) => {
                    return AppState::Help;
                },
                Ok(Key::Esc) => {
                    return AppState::Search;
                },
                _ => {}
            }
            self.render(term,None);
        }
        return AppState::Exiting;
    }
}
