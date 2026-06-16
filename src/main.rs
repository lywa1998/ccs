use std::collections::HashMap;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::{self, Command};

use ratatui::style::Color;

mod config;
mod preview;
mod tui;

use config::{load_config, resolve_profile};

pub(crate) const ACCENT: Color = Color::Cyan;
pub(crate) const MUTED: Color = Color::DarkGray;

pub(crate) fn fatal(msg: &str) -> ! {
    eprintln!("\x1b[31merror\x1b[0m: {msg}");
    process::exit(1);
}

fn launch(profile: &config::Profile) -> ! {
    let mut env_map: HashMap<String, String> = env::vars()
        .filter(|(k, _)| !k.starts_with("ANTHROPIC_"))
        .collect();

    env_map.extend(config::build_env(profile, true));

    let user_args: Vec<String> = env::args()
        .skip(1)
        .filter(|a| a != "--dangerously-skip-permissions")
        .collect();
    let mut args = vec!["--dangerously-skip-permissions".to_string()];
    args.extend(user_args);

    let err = Command::new("claude")
        .args(&args)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .env_clear()
        .envs(&env_map)
        .exec();

    fatal(&format!("failed to launch claude: {err}"));
}

fn main() {
    let config = load_config();
    let app = tui::App::new(config);
    let (_app, selected) = tui::run(app);
    match selected {
        Some(name) => {
            let config = load_config();
            match resolve_profile(&config, &name) {
                Ok(profile) => launch(&profile),
                Err(e) => fatal(&e),
            }
        }
        None => {
            println!("no profile selected, exiting");
        }
    }
}
