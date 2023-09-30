# Workspacer
A basic command line tool for creating preset 'workspaces' that can be easily started with a single command.

## Building
To build workspacer run the following command (You will need cargo and rust installed):
`cargo build`

## Usage
*For best usage place workspacer.exe into a directory and then add it to your PATH environment variable. It is probably a good idea to rename it to something shorter such as `w`*

__To open a workspacer run `workspacer WORKSPACENAME`__

See the example_config.toml for an example of how workspaces are formatted.
The config should be located in your pc config location, for windows this is `./appdata/local/workspacer/config.toml`

## Commands
- `-list`: Lists the names of your workspaces
- `-new`: Creates a new workspace
- 'ANYINPUT': Any text input that is not a command will be interpreted as a workspace name that it will attempt to open
