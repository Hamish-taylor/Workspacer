use dirs;
use std::env;
use std::fs;
use std::io::stdin;
use std::io::Error;
use std::io::{self, stdout, Write};
use std::process::Command;
use toml;

use crossterm::{cursor, ExecutableCommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    workspace: Vec<Workspace>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Workspace {
    WorkspaceVariant { name: String, tab: Vec<Tab> },
    GroupVariant { name: String, group: Vec<String> },
}

impl Workspace {
    fn name(&self) -> &String {
        match self {
            Workspace::WorkspaceVariant { name, .. } => name,
            Workspace::GroupVariant { name, .. } => name,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Tab {
    title: Option<String>,
    starting_directory: String,
    commands: Option<Vec<String>>,
    split_pane: Option<bool>,
}

fn get_workspace(config: &Config, workspace_string: String) -> Option<&Workspace> {
    config
        .workspace
        .iter()
        .find(|workspace| workspace.name().to_lowercase() == workspace_string.to_lowercase())
}

fn open_workspace(config: &Config, workspace: &Workspace) {
    match workspace {
        Workspace::GroupVariant { name, group } => {
            for workspace_name in group {
                let w = get_workspace(config, workspace_name.clone()).unwrap();
                open_workspace(&config, &w);
            }
        }
        Workspace::WorkspaceVariant { name, tab } => {
            let mut command = Command::new("wt");
            for (i, tab) in tab.iter().enumerate() {
                if i > 0 {
                    command.arg(";");
                    if let Some(split_pane) = &tab.split_pane {
                        if *split_pane {
                            command.arg("split-pane");
                        } else {
                            command.arg("new-tab");
                        }
                    } else {
                        command.arg("new-tab");
                    }
                }
                if let Some(title) = &tab.title {
                    command.arg("--title").arg(title);
                }
                command
                    .arg("--startingDirectory")
                    .arg(&tab.starting_directory);
                if let Some(commands) = &tab.commands {
                    for c in commands {
                        command.arg(&c);
                    }
                }
            }

            match command.status() {
                Ok(status) => {
                    if status.success() {
                        println!("Successfully opened Windows Terminal with tabs.");
                    } else {
                        eprintln!("Command executed with error: {:?}", status);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute command: {}", e);
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let primary_arg = args.get(1);

    if let Some(config_path) = get_config_path() {
        let mut config: Config = match load_config(&config_path) {
            Ok(inner_workspace) => inner_workspace,
            Err(err) => {
                println!("No config found: {:?}", err);
                return;
            }
        };

        if let Some(value) = primary_arg {
            match value.to_lowercase().as_str() {
                "-new" => {
                    config
                        .workspace
                        .push(create_new_workspace_command().unwrap());
                    save_config(&config, &config_path).unwrap();
                }
                "-list" => {
                    for workspace in config.workspace {
                        println!("{}", workspace.name());
                    }
                }
                w => {
                    let mut command = Command::new("wt");
                    let workspace: &Workspace = get_workspace(&config, w.into()).unwrap();
                    open_workspace(&config, workspace);
                }
                _ => {
                    println!("Unknow command");
                }
            }
        }
    }
}

fn create_new_workspace_command() -> Result<Workspace, Error> {
    let mut stdout = stdout();

    let mut name = String::new();

    stdout.execute(cursor::Show)?;

    print!("Enter workspace name: ");
    stdout.flush()?;

    read_input(&mut name)?;

    let mut tabs = vec![];
    loop {
        let mut title = String::new();
        print!("Enter title for the tab: ");
        stdout.flush()?;
        read_input(&mut title)?;

        let mut starting_directory = String::new();
        print!("Enter starting directory for the Tab: ");
        stdout.flush()?;
        read_input(&mut starting_directory)?;

        let mut commands = String::new();
        print!("Enter commands for the tab: ");
        stdout.flush()?;
        read_input(&mut commands)?;

        let mut split_pane_str = String::new();
        let mut split_pane = false;
        loop {
            print!("Tab should be a split pane? y/n: ");
            stdout.flush()?;
            read_input(&mut split_pane_str)?;

            if split_pane_str.trim().eq_ignore_ascii_case("y") {
                split_pane = true;
                break;
            } else {
                break;
            }
        }
        // Further questions can be added as needed...
        // ...

        let tab = Tab {
            title: if title.is_empty() { None } else { Some(title) },
            starting_directory,
            commands: Some(commands.split(" ").map(|a| a.to_string()).collect()),
            split_pane: Some(split_pane),
        };
        tabs.push(tab);
        print!("Would you like to make another tab y/n: ");
        let mut ctn = String::new();
        stdout.flush()?;
        read_input(&mut ctn)?;

        println!("_{}_", ctn);
        if !ctn.trim().eq_ignore_ascii_case("y") {
            println!("{}", ctn);
            break;
        }
    }

    let workspace = Workspace::WorkspaceVariant {
        name,
        tab: tabs,
    };

    // Now, use the created workspace...
    println!("Created Workspace: {:?}", workspace);

    Ok(workspace)
}

fn read_input(input: &mut String) -> io::Result<bool> {
    input.clear();
    stdin().read_line(input)?;
    if input.trim().eq_ignore_ascii_case("q") {
        return Ok(true);
    }
    *input = input
        .trim()
        .trim_end_matches("\n")
        .trim_end_matches("\r")
        .to_string();
    Ok(false)
}

/*
fn init() -> bool {
    let mut input = String::new();

    println!("Please enter a path for your first workspace \n(note that aditional workspaces can be added using the 'workspacer inner_workspace <path>' command)");

    match io::stdin().read_line(&mut input) {
        Err(err) => {
            panic!("Error reading from standard in: {:?}", err)
        }
        Ok(_) => (),
    }
    input = input.replace("\r", "");
    input = input.replace("\n", "");
    input = input.trim().to_owned();

    let path = std::path::PathBuf::from(input);

    //check that the path exists
    if let Ok(false) = path.try_exists() {
        println!("Supplied path {:?} is not valid", path);
        return false;
    }

    print!("{:?}", path);

    let config_path = match get_config_path() {
        Some(v) => v,
        None => panic!("error fetching systems config path"),
    };

    let parent_dir = match config_path.parent() {
        Some(v) => v,
        None => {
            println!(
                "Could not get parent dir from config path: {:?}",
                config_path
            );
            return false;
        }
    };

    match fs::create_dir_all(parent_dir) {
        Ok(_) => println!("Created dir"),
        Err(err) => {
            println!("Failed to create config dir: {:?} - {:?}", parent_dir, err)
        }
    }
    let workspace = Workspace {
        name: "".to_owned(),
        path: path.to_str().unwrap().to_owned(),
    };

    let config = Config {
        workspaces: vec![workspace],
    };
    match save_config(&config, &config_path) {
        Ok(_) => println!("Default config saved"),
        Err(err) => println!("Error trying to create a default config: {:?}", err),
    }

    true
}
*/

fn get_config_path() -> Option<std::path::PathBuf> {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("Workspacer"); // replace 'your_app_name' with your actual app name
        config_dir.push("config.toml");
        Some(config_dir)
    } else {
        None
    }
}

fn load_config(path: &std::path::Path) -> Result<Config, toml::de::Error> {
    let data = fs::read_to_string(path).unwrap();
    let config: Config = toml::from_str(&data)?;
    Ok(config)
}

fn save_config(config: &Config, path: &std::path::Path) -> Result<(), Error> {
    let content = toml::to_string_pretty(config)
        .map_err(|e: toml::ser::Error| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, content)?;
    Ok(())
}
