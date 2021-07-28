use std::io;
use shiplift::Docker;
use termion::{raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    Terminal,
};
use termion::color;
use serde::{Deserialize,Serialize};
use serde_json;
use std::time::SystemTime;
use std::path::Path;
use home;

mod ui;

const VERSION: &'static str = concat!("Docker development environment version v",env!("CARGO_PKG_VERSION"));

#[derive(Serialize,Deserialize)]
struct Configuration {
    images_last_modified: u64
}


fn enter_to_start() {
    println!("Press enter to start program...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
}

fn has_changed_config_file(path: &Path) -> bool {
    let configuration = match std::fs::read_to_string(path.join(".config")) {
        Ok(cfg) => {
            serde_json::from_str::<Configuration>(&cfg).expect(&format!("Could not deserialize .config file from directory {:?}, file is corrupted delete it to recover!",path))
        },
        _ => {
            Configuration {
                images_last_modified: 0,
            }
        }
    };
    let mut last_modified_seconds = 0;
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        last_modified_seconds = std::cmp::max(entry.path().metadata().unwrap().modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),last_modified_seconds);
    }
    configuration.images_last_modified < last_modified_seconds
}

fn update_config_file(path: &Path) {
    let config = Configuration {
        images_last_modified: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
    };
    std::fs::write(path.join(".config"), &serde_json::to_string(&config).unwrap()).unwrap();
}

pub fn install_self() {
    let install_path = "/bin/dde";
    let exe_path = std::env::current_exe().expect("Could not get executable path!");
    std::fs::copy(exe_path,install_path).expect("Could not install executable to binary directory");
    println!("Installation sucessful!");
}

fn print_error(error: &str) {
    println!("{}{}{}",color::Fg(color::LightRed),error,color::Fg(color::Reset));
}

async fn write_basic_setup() -> Docker {
    let docker = Docker::new();
    let basic_vim_rc = std::include_str!("../config/basic/init.vim");
    let basic_dockerfile = std::include_str!("../config/basic/dockerfile");
    let basic_zshrc = std::include_str!("../config/basic/.zshrc");
    let basic_p10k = std::include_str!("../config/basic/.p10k.zsh");
    let help_file = std::include_str!("../config/basic/help.md");

    if let Some(home) = home::home_dir() {
        let base_dir = home.join(".docker_development_env");
        let _ = std::fs::create_dir(&base_dir);
        
        let basic_image = base_dir.join("basic");
        if !basic_image.exists() {
            println!("Could not find basic image, writing it now!");
            std::fs::create_dir(&basic_image).unwrap();
            std::fs::write(basic_image.join("init.vim"), basic_vim_rc).unwrap();
            std::fs::write(basic_image.join("dockerfile"), basic_dockerfile).unwrap();
            std::fs::write(basic_image.join(".zshrc"), basic_zshrc).unwrap();
            std::fs::write(basic_image.join(".p10k.zsh"), basic_p10k).unwrap();
            std::fs::write(basic_image.join("help.md"), help_file).unwrap();
        }
        let new_path = basic_image.parent().unwrap();
        println!("{}Updating all images in {:?}.{}",color::Fg(color::Green),new_path,color::Fg(color::Reset));
        if new_path.exists() && new_path.is_dir() {
            let mut changed_something = false;
            for entry in std::fs::read_dir(new_path).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if !path.is_dir() {
                    println!("Found file {:?}. Skipping...",path);
                }
                else {
                    let dockerfile = path.join("dockerfile");
                    if !dockerfile.exists() {
                        changed_something = true;
                        print_error(&format!("Could not find dockerfile for base image {:?}! Skipping...",path.file_name().unwrap()));
                    }
                    else {
                        if has_changed_config_file(&path) {
                            changed_something = true;
                            println!("{}Building image from base image directory {:?}...{}",color::Fg(color::Green),path.file_name().unwrap(),color::Fg(color::Reset));
                            let path_file : &str = path.file_name().unwrap().to_str().unwrap();
                            println!("build path {}",path.to_str().unwrap());
                            let result = std::process::Command::new("docker").args(vec!["build",path.to_str().unwrap(),"-t",&(String::from("dde_")+path_file)]).spawn().unwrap().wait();
                            if result.is_ok() && result.unwrap().success() {
                                update_config_file(&path);
                                println!("Build the image sucessfully.");
                            }
                            else {
                                print_error("Some errors where encountered please fix them and try again!");
                            }
                        }
                        else {
                            println!("Skipping {:?} no changes found in this directory.",path);
                        }
                    }
                }
            }
            if changed_something {
                enter_to_start();
            }
        }
    }
    else {
        println!("{}Could not find home directory. The program runs without base images!{}",color::Fg(color::Red),color::Fg(color::Reset));
        enter_to_start();
    }
    docker
}

pub fn get_base_dir() -> Option<String> {
    home::home_dir().filter(|x| {
        let result = x.join(".docker_development_env");
        result.exists() && result.is_dir()
    }).map(|x| x.join(".docker_development_env").to_str().unwrap().to_string())
}

#[tokio::main]
async fn main() {
    println!("{}{}{}",color::Fg(color::LightMagenta),VERSION,color::Fg(color::Reset));
    let args: Vec<String> = std::env::args().collect();
    if let Some(command) = args.get(1) {
        if command == "install" {
            install_self();
            return;
        }
        else if command == "add" {
            if args.len() < 3 {
                print_error("Too few arguments what directory or git repository should be added?");
                std::process::exit(-1);
            }
            let folder = &args[2];
            let path = std::path::Path::new(&folder);
            if let Some(base) = get_base_dir() {
                let file = path.join("dockerfile");
                if path.exists() && path.is_dir() {
                    println!("Trying to add {:?} to base images",path);

                    if file.exists() && file.is_file() {
                        println!("orig {:?} link {:?}",path,std::path::Path::new(&base).join(path.file_name().unwrap()));
                        std::os::unix::fs::symlink(std::fs::canonicalize(path).unwrap(),std::path::Path::new(&base).join(path.file_name().unwrap())).unwrap();
                        println!("Added the repository {:?} to the base images directory as link!",path);
                    }
                    else {
                        print_error(&format!("Could not find docker file {:?}! Skipped adding!",file));
                        std::process::exit(-1);
                    }
                }
                else {
                    println!("Cannot find path {:?} attempting cloning from git!",path);
                    if args.len() < 4 {
                        print_error("Too few arguments for a git repository the name of the image is missing!");
                        std::process::exit(-1);
                    }
                    let full_path = std::path::Path::new(&base).join(&args[3]);
                    if let Ok(result) = std::process::Command::new("git").args(vec!["clone",path.to_str().unwrap(),full_path.to_str().unwrap()]).spawn().unwrap().wait() {
                        if result.success() {
                            println!("Cloned the directory sucessfully!");
                            if !full_path.join("dockerfile").exists() || full_path.join("dockerfile").is_file(){
                                print_error(&format!("The repository {} does not contain a docker file removing it...",folder));
                                std::fs::remove_dir_all(full_path).unwrap();
                                std::process::exit(-1);
                            }
                        }
                    }
                }
            }
            else {
                print_error(&format!("Could not find image base directory can't add image!"));
                std::process::exit(-1);
            }
        }
        else if command == "config" {
            if let Some(base) = get_base_dir() {
                println!("Base image directory: {}",base);
                return;
            }
            else {
                print_error("Cannot find base directory!");
                std::process::exit(-1);
            }
        }
    }
    let docker = write_basic_setup().await;
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    let mut app = ui::App::new(docker).await;
    app.event_loop(&mut terminal).await;
}
