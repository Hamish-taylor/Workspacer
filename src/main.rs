use dirs;
use std::env;
use std::fs;
use std::io::Error;
use std::io::{self, stdout, Write};
use toml;

use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, ExecutableCommand};

fn print_directories(
    dir: &str,
    selected: usize,
    offset: usize,
    terminal_height: usize,
    filter_string: &String,
) -> io::Result<Vec<String>> {
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

fn list_files(dir: &str) -> io::Result<(String)> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    stdout().execute(terminal::Clear(ClearType::All))?;

    let terminal_size = terminal::size()?;
    let terminal_height = terminal_size.1 as usize; // Height is the second element of the tuple

    let mut stdout = stdout();
    stdout.execute(cursor::Hide)?;

    let mut selected = 0;
    let mut offset = 0;

    let mut filter_string = String::new();

    let mut last_visible: Vec<String> = vec![];
    stdout.flush()?;
    loop {
        let visible_directories =
            print_directories(dir, selected, offset, terminal_height, &filter_string)?;

        // Update changed lines and keep track of the longest line for clearing purposes
        let mut max_len = 0;
        for (i, (last, new)) in last_visible.iter().zip(&visible_directories).enumerate() {
            max_len = max_len.max(last.len());
            if last != new {
                stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
                print!("{}\r", " ".repeat(last.len()));
                stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
                print!("{}", new);
            }
        }

        // If the new list is shorter than the last, clear the remaining lines
        for i in visible_directories.len()..last_visible.len() {
            stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
            print!("{}\r", " ".repeat(max_len));
        }

        // If the new list is longer than the last, print the new lines
        for i in last_visible.len()..visible_directories.len() {
            stdout.execute(cursor::MoveTo(0, (i + 1) as u16))?;
            print!("{}", visible_directories[i]);
        }

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
                            if selected < offset {
                                offset -= 1;
                            }
                        }
                    }
                    (event::KeyCode::Down, _) => {
                        selected += 1;
                        if selected >= offset + terminal_height {
                            // Assuming we are showing 10 at a time
                            offset += 1;
                        }
                    }
                    (event::KeyCode::Right, _) | (event::KeyCode::Enter, _) => {
                        stdout.execute(cursor::Show)?;
                        terminal::disable_raw_mode()?;
                        execute!(io::stdout(), LeaveAlternateScreen);
                        return Ok(last_visible
                            .clone()
                            .get(selected - offset)
                            .unwrap()
                            .replace("> ", ""));
                    }
                    (event::KeyCode::Backspace, _) => {
                        filter_string.pop();
                    }
                    (event::KeyCode::Char(char), _) => {
                        filter_string.push(char);
                        selected = 0;
                        io::stdout().execute(terminal::Clear(ClearType::All))?;
                    }
                    _ => {}
                }
            }
        }
    }

    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen);
    Ok("".to_owned())
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    workspaces: Vec<Workspace>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Workspace {
    name: String,
    path: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args.to_owned());
    let primary_arg = args.get(1);

    if let Some(config_path) = get_config_path() {
        let config: Config = match load_config(&config_path) {
            Ok(config) => {
                println!("Loaded config: {:?}", config);
                config
            }
            Err(_) => {
                println!("No config found, running init");
                let mut initialized = false;

                while !initialized {
                    initialized = init();
                }
                return;
            }
        };
        //clear the file path
        let mut file_path = config_path.clone().parent().unwrap().to_path_buf();
        file_path.push("file.txt");
        println!("Saving config to: {:?}", file_path);
        fs::write(file_path, "");

        println!("{:?}", config.workspaces.get(0).unwrap().path.as_str());
        if let Some(value) = primary_arg {
            match value.to_lowercase().as_str() {
                //"init" | "i" => init(),
                "list" | "lf" | "l" => {
                    match list_files(config.workspaces.get(0).unwrap().path.as_str()) {
                        Ok(file) => {
                            if file != "".to_owned() {
                                println!("Selected: {:?}", file);

                                //Theoretically check that the file exists
                                //Then write it to a file
                                let workspace_path = match config.workspaces.get(0) {
                                    Some(v) => v.path.to_owned(),
                                    None => panic!("Error"),
                                };
                                println!("workspace_path: {:?}", workspace_path);
                                let folder_dir_to_open = workspace_path + "/" + file.as_str();
                                let mut path = config_path.parent().unwrap().to_path_buf();
                                path.push("file.txt");
                                println!("Saving file {:?} to {:?}", folder_dir_to_open, path);
                                match fs::write(path, folder_dir_to_open) {
                                    Err(err) => println!("Error, {:?}", err),
                                    _ => {}
                                }
                            } else {
                                println!("err");
                            }
                        }
                        Err(err) => println!("Encountered error when selecting file: {:?}", err),
                    }
                }
                file => {
                    println!("{:?}", String::from("opening: ") + file);

                    //Theoretically check that the file exists
                    //Then write it to a file
                    let workspace_path = config.workspaces.get(0).unwrap().path.to_owned();
                    let folder_dir_to_open = workspace_path + "/" + file;
                    let mut path = config_path.parent().unwrap().to_path_buf();
                    path.push("file.txt");
                    fs::write(path, folder_dir_to_open).unwrap();
                }
                _ => println!("Unknow arg"),
            }
        }
    }
}

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

fn get_config_path() -> Option<std::path::PathBuf> {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("Workspacer"); // replace 'your_app_name' with your actual app name
        config_dir.push("config.toml");
        Some(config_dir)
    } else {
        None
    }
}

fn load_config(path: &std::path::Path) -> Result<Config, Error> {
    println!("Loading config from: {:?}", path);

    let data = fs::read_to_string(path)?;

    let config: Config = toml::from_str(&data)
        .map_err(|e: toml::de::Error| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(config)
}

fn save_config(config: &Config, path: &std::path::Path) -> Result<(), Error> {
    println!("Saving config to: {:?}", path);
    let content = toml::to_string_pretty(config)
        .map_err(|e: toml::ser::Error| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, content)?;
    Ok(())
}
