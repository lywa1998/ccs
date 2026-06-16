# cc

Interactive profile switcher for [Claude Code](https://claude.ai/code).  
Quickly switch between different API providers and model configurations via a terminal UI.

## Install

```bash
cargo install --git https://github.com/lywa1998/cc.git
```

## Configuration

Profiles are defined in `~/.config/cc/config.toml`:

```toml
[profiles.deepseek]
description = "DeepSeek"

[profiles.deepseek.models]
default = "deepseek-v4-pro[1m]"
default_fable = "deepseek-v4-pro[1m]"

[profiles.deepseek.provider]
base_url = "https://api.deepseek.com/anthropic"
env_key = "DEEPSEEK_API_KEY_CLAUDE_CODE"

[profiles.bytecat]
description = "ByteCat GPT"

[profiles.bytecat.models]
default = "gpt-5.4"
small_fast = "gpt-5.4-mini"

[profiles.bytecat.provider]
base_url = "https://bytecat.example.com"
env_key = "BYTECAT_API_KEY"
```

### Model fields

All model fields are optional. Unset fields are simply not passed to Claude Code, so its built-in defaults apply.

| Field | Environment variable | Purpose |
|-------|---------------------|---------|
| `default` | `ANTHROPIC_MODEL` | Main model |
| `default_fable` | `ANTHROPIC_DEFAULT_FABLE_MODEL` | Fable 5 for hardest tasks |
| `default_opus` | `ANTHROPIC_DEFAULT_OPUS_MODEL` | Opus for complex reasoning |
| `default_sonnet` | `ANTHROPIC_DEFAULT_SONNET_MODEL` | Sonnet for daily coding |
| `default_haiku` | `ANTHROPIC_DEFAULT_HAIKU_MODEL` | Haiku for simple/background tasks |

### Provider fields

| Field | Environment variable | Purpose |
|-------|---------------------|---------|
| `base_url` | `ANTHROPIC_BASE_URL` | API endpoint URL |
| `env_key` | — | Name of the env var that holds the API key |

## Usage

```bash
cc              # open the TUI profile picker
cc <profile>    # launch directly with a profile
```

### Keybindings

| Key | Context | Action |
|-----|---------|--------|
| `↑` `↓` / `j` `k` | Left panel | Navigate profiles |
| `Enter` | Left panel | Select profile & launch Claude |
| `n` | Left panel | Create new profile |
| `→` | Left panel | Focus right panel (edit fields) |
| `←` | Right panel | Focus left panel |
| `Enter` | Right panel | Edit selected field |
| `↑` `↓` / `j` `k` | Right panel | Navigate fields |
| `Enter` | Editing | Confirm & save |
| `Esc` | Editing / Creating | Cancel |
| `q` / `Esc` / `Ctrl-c` | Global | Quit |

## License

MIT
