# cc

Interactive profile switcher for [Claude Code](https://claude.ai/code).  
Quickly switch between different API providers and model configurations via a terminal UI.

## Install

```bash
cargo install --git https://github.com/lywa1998/cc.git
```

Or use `cargo binstall` from GitHub Releases:

```bash
cargo binstall --git https://github.com/lywa1998/cc.git cc
```

## Configuration

Profiles are defined in `~/.config/cl/config.toml`:

```toml
[profiles.deepseek]
description = "DeepSeek"

[profiles.deepseek.models]
default = "deepseek-v4-pro[1m]"
default_fable = "deepseek-v4-pro[1m]"

[profiles.deepseek.provider]
base_url = "https://api.deepseek.com/anthropic"
env_key = "DEEPSEEK_API_KEY_CLAUDE_CODE"

[profiles.other]
description = "Another provider"
extends = "deepseek"           # inherit models from deepseek

[profiles.other.models]
default = "gpt-5.4"

[profiles.other.provider]
base_url = "https://other.example.com"
env_key = "OTHER_API_KEY"
```

## Usage

```bash
cc              # open the TUI profile picker
cc <profile>    # launch directly with a profile
```

### Keybindings

| Key | Action |
|-----|--------|
| `↑` `↓` / `j` `k` | Navigate profiles |
| `Enter` | Select profile & launch Claude Code |
| `q` / `Esc` / `Ctrl-c` | Quit |

## License

MIT
