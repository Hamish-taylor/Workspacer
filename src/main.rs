use dirs;
use std::env;
use std::fs;
use std::io::stdin;
use std::io::Error;
use std::io::{self, stdout, Write};
use std::process::Command;
use toml;

use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, ExecutableCommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    workspace: Vec<Workspace>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Workspace {
    name: String,
    tab: Vec<Tab>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Tab {
    title: Option<String>,
    starting_directory: String,
    commands: Option<Vec<String>>,
    split_pane: Option<bool>,
}

struct TerminalLine {}

fn print_directories(
    directories: &Vec<String>,
    selected: usize,
    offset: usize,
    terminal_height: usize,
    filter_string: &String,
) -> io::Result<Vec<String>> {
    let end = std::cmp::min(offset + terminal_height - 2, directories.len());
    let visible_directories: Vec<String> = directories
        .iter()
        .enumerate()
        .skip(offset)
        .take(end - offset)
        .map(|(index, dir)| {
            if index == selected {
                format!("> {}", dir)
            } else {
                format!("  {}", dir)
            }
        })
        .collect();

    Ok(visible_directories)
}

fn list_files(dir: &str) -> io::Result<String> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    stdout().execute(terminal::Clear(ClearType::All))?;

    let terminal_size = terminal::size()?;
    let terminal_height = terminal_size.1 as usize; // Height is the second element of the tuple

    let mut stdout = stdout();

    let mut selected = 0;
    let mut offset = 0;

    let mut filter_string = String::new();

    let mut last_visible: Vec<String> = vec![];
    let mut old_filter_string = String::new();
    stdout.flush()?;

    let paths = fs::read_dir(dir)?;

    let mut directories = vec![];

    for path in paths {
        if let Ok(entry) = path {
            let file_name = match entry.file_name().into_string() {
                Ok(v) => v.to_lowercase(),
                Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "")),
            };
            if entry.file_type().unwrap().is_dir() && file_name.starts_with(filter_string.as_str())
            {
                directories.push(entry.file_name().into_string().unwrap());
            }
        }
    }

    loop {
        let visible_directories = print_directories(
            &directories,
            selected,
            offset,
            terminal_height - 2,
            &filter_string,
        )?;

        //if new search string is longer
        //  print new chars
        //
        //if new search string is shorter
        //  clear the old chars

        // Update changed lines and keep track of the longest line for clearing purposes

        stdout.execute(cursor::Hide)?;
        let mut max_len = 0;
        for (i, (last, new)) in last_visible.iter().zip(&visible_directories).enumerate() {
            max_len = max_len.max(last.len());
            if last != new {
                if (i + offset) == selected {
                    stdout.execute(SetBackgroundColor(Color::Blue))?;
                }
                stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
                print!("{}\r", " ".repeat(terminal_size.0 as usize));
                stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
                print!("{}", new);
                stdout.execute(ResetColor)?;
            }
        }

        // If the new list is shorter than the last, clear the remaining lines
        for i in visible_directories.len()..last_visible.len() {
            stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
            print!("{}\r", " ".repeat(terminal_size.0 as usize));
        }

        // If the new list is longer than the last, print the new lines
        for i in last_visible.len()..visible_directories.len() {
            if (i + offset) == selected {
                stdout.execute(SetBackgroundColor(Color::Blue))?;
            }
            stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
            print!("{}", visible_directories[i]);
            print!("{}\r", " ".repeat(terminal_size.0 as usize));
            stdout.execute(ResetColor)?;
        }

        stdout.execute(cursor::Show)?;
        match render_search_text(terminal_height, &old_filter_string, &filter_string) {
            Err(err) => panic!("{}", err),
            _ => {}
        };

        stdout.flush()?;

        last_visible = visible_directories;
        if let event::Event::Key(event::KeyEvent {
            code,
            kind,
            modifiers,
            state,
        }) = event::read()?
        {
            if kind == KeyEventKind::Press {
                match (code, modifiers) {
                    (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL) => break,
                    (event::KeyCode::Up, _) => {
                        if selected > 0 {
                            selected -= 1;
                            if selected - 7 < offset {
                                offset -= 1;
                            }
                        }
                    }
                    (event::KeyCode::Down, _) => {
                        if selected + 1 < directories.len() {
                            selected += 1;
                            if selected >= offset + terminal_height - 8 {
                                // Assuming we are showing 10 at a time
                                offset += 1;
                            }
                        }
                    }
                    (event::KeyCode::Right, _) | (event::KeyCode::Enter, _) => {
                        stdout.execute(cursor::Show)?;
                        terminal::disable_raw_mode()?;
                        execute!(io::stdout(), LeaveAlternateScreen)?;
                        return Ok(last_visible
                            .clone()
                            .get(selected - offset)
                            .unwrap()
                            .replace("> ", ""));
                    }
                    (event::KeyCode::Backspace, _) => {
                        old_filter_string = filter_string.clone();
                        filter_string.pop();
                        selected = 0;
                    }
                    (event::KeyCode::Char(char), _) => {
                        old_filter_string = filter_string.clone();
                        filter_string.push(char);
                        selected = 0;
                    }
                    _ => {}
                }
            }
        }
    }

    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok("".to_owned())
}

fn render_search_text(
    terminal_height: usize,
    old_filter_string: &String,
    filter_string: &String,
) -> Result<(), Error> {
    let old_count = old_filter_string.chars().count();
    let new_count = filter_string.chars().count();
    if old_count < new_count {
        for (i, c) in filter_string.chars().skip(old_count).enumerate() {
            io::stdout().execute(cursor::MoveTo(
                (old_count + i) as u16,
                terminal_height as u16,
            ))?;
            print!("{}", c);
        }
    } else if old_count > new_count {
        io::stdout().execute(cursor::MoveTo(new_count as u16, terminal_height as u16))?;
        print!("{}", " ".repeat(old_count - new_count));
        io::stdout().execute(cursor::MoveTo(
            filter_string.chars().count() as u16,
            terminal_height as u16,
        ))?;
    } else {
        io::stdout().execute(cursor::MoveTo(0 as u16, terminal_height as u16))?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let primary_arg = args.get(1);

    if let Some(config_path) = get_config_path() {
        let mut config: Config = match load_config(&config_path) {
            Ok(w) => {
                w
            }
            Err(err) => {
                println!("No config found: {:?}", err);
                return;
            }
        };

        if let Some(value) = primary_arg {
            match value.to_lowercase().as_str() {
                "-new" => {
                    config.workspace.push(create_new_workspace_command().unwrap());
                    save_config(&config, &config_path).unwrap();
                }
                "-list" => {
                    for workspace in config.workspace {
                        println!("{}", workspace.name);
                    }
                }
                w => {
                    let mut command = Command::new("wt");
                    let workspace: &Workspace = config
                        .workspace
                        .iter()
                        .find(|workspace| workspace.name.to_lowercase() == w.to_lowercase())
                        .unwrap();
                    for (i, tab) in workspace.tab.iter().enumerate() {
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

        println!("_{}_",ctn);
        if !ctn.trim().eq_ignore_ascii_case("y") {
            println!("{}",ctn);
            break;
        }  
    }

    let workspace = Workspace { name, tab: tabs };

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
    *input =  input.trim().trim_end_matches("\n").trim_end_matches("\r").to_string();
    Ok(false)
}

/*
fn init() -> bool {
    let mut input = String::new();

    println!("Please enter a path for your first workspace \n(note that aditional workspaces can be added using the 'workspacer w <path>' command)");

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
