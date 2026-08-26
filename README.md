# wam

A simple CLI and TUI web app manager
It automatically fetches icons and created .desktop files

## TODO

- [x] Add CLI functionality
  - [x] add command
  - [x] remove command
  - [x] list command
  - [x] sync command
  - [x] toggle command
- [ ] Add TUI functionality
  - [ ] Main menu
  - [ ] Add menu
  - [ ] Edit menu
  - [ ] Settings menu

## Installation

### crates.io

```bash
# Make sure $CARGO_HOME/bin is in $PATH

cargo install wam-manager
```

### Nix

```bash
nix run github:bardiya01/wam
```

### Build from source

```bash
git clone https://github.com/bardiya01/wam.git

cd wam

cargo build --release

./target/release/wam
```

## USAGE

```bash
Usage: wam [OPTIONS] [COMMAND]

Commands:
  add     Add a new web-app
  remove  Remove a web-app
  list    List web-apps
  sync    Sync all configured web-apps
  toggle  Toggle the .desktop file
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <FILE>  Path to a config file
  -h, --help           Print help
  -V, --version        Print version
```

> [!TIP]
> You can run wam sync on a new install to fetch all icons
> and generate all .desktop files

## Config

### Locations

1. --config FILE
2. $WAM_CONFIG_FILE
3. $XDG_CONFIG_HOME/wam/config.toml
4. $HOME/.config/wam/config.toml

```toml
[settings]
browser = "helium" # Browser preset, list below
custom_command = "helium --app={app} --no-first-run --no-default-browser-check"
desktop_file_dir = "/path/to/desktop/files" # Defaults to $XDG_DATA_HOME/applications/
icon_file_dir = "/path/to/icon/files" # Defaults to $XDG_CACHE_HOME/wam/icons/

# A list of apps

[apps.monkeytype] # Name
url = "https://monkeytype.com/" # URL
status = "enabled" # if disabled will not have .desktop in the desktop_file_dir

[apps.Github]
url = "https://github.com"
status = "enabled"
```

### Browser presets

| Browser          | Command Template                                               |
| ---------------- | -------------------------------------------------------------- |
| `helium`         | `helium --app={app} --no-first-run --no-default-browser-check` |
| `chrome`         | `google-chrome --app={app}`                                    |
| `chromium`       | `chromium --app={app}`                                         |
| `brave`          | `brave --app={app}`                                            |
| `microsoft-edge` | `microsoft-edge --app={app}`                                   |
| `vivaldi`        | `vivaldi --app={app}`                                          |
| `firefox`        | `firefox --new-window {app}`                                   |
| `librewolf`      | `librewolf --new-window {app}`                                 |
