use std::collections::HashMap;
use std::env;

use serde::Deserialize;
use url::Url;

// ── types ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Clone)]
pub struct Provider {
    pub base_url: Url,
    pub env_key: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct Models {
    pub default: Option<String>,
    pub small_fast: Option<String>,
    pub default_haiku: Option<String>,
    pub default_sonnet: Option<String>,
    pub default_opus: Option<String>,
}

#[derive(Deserialize, Clone, Default)]
pub struct Profile {
    pub extends: Option<String>,
    pub description: Option<String>,
    pub models: Option<Models>,
    pub provider: Option<Provider>,
}

#[derive(Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
}

// ── config loading ─────────────────────────────────────────────────────────

fn config_path() -> String {
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))
        .expect("cannot determine home directory");
    format!("{home}/.config/cl/config.toml")
}

pub fn load_config() -> Config {
    let cpath = config_path();
    let content = std::fs::read_to_string(&cpath)
        .unwrap_or_else(|e| fatal(&format!("cannot read {cpath}: {e}")));
    let config: Config = toml::from_str(&content)
        .unwrap_or_else(|e| fatal(&format!("invalid config in {cpath}: {e}")));
    if config.profiles.is_empty() {
        fatal("no profiles defined in config");
    }
    config
}

// ── profile resolution ─────────────────────────────────────────────────────

pub fn resolve_profile(config: &Config, name: &str) -> Result<Profile, String> {
    let mut chain: Vec<&str> = Vec::new();
    let mut current: Option<&str> = Some(name);

    while let Some(cur) = current {
        if chain.contains(&cur) {
            chain.push(cur);
            return Err(format!("circular extends: {}", chain.join(" -> ")));
        }
        let profile = config.profiles.get(cur).ok_or_else(|| {
            if chain.is_empty() {
                format!("unknown profile \"{cur}\"")
            } else {
                format!(
                    "profile \"{}\" extends unknown profile \"{cur}\"",
                    chain.last().unwrap()
                )
            }
        })?;
        chain.push(cur);
        current = profile.extends.as_deref();
    }

    let mut merged = Profile::default();
    for name in chain.iter().rev() {
        let profile = &config.profiles[*name];
        if let Some(ref src) = profile.models {
            let dst = merged.models.get_or_insert_default();
            if let Some(ref v) = src.default { dst.default = Some(v.clone()); }
            if let Some(ref v) = src.small_fast { dst.small_fast = Some(v.clone()); }
            if let Some(ref v) = src.default_haiku { dst.default_haiku = Some(v.clone()); }
            if let Some(ref v) = src.default_sonnet { dst.default_sonnet = Some(v.clone()); }
            if let Some(ref v) = src.default_opus { dst.default_opus = Some(v.clone()); }
        }
        if profile.provider.is_some() {
            merged.provider = profile.provider.clone();
        }
    }

    Ok(merged)
}

// ── environment ────────────────────────────────────────────────────────────

pub fn build_env(profile: &Profile, reveal: bool) -> HashMap<String, String> {
    let mut env_map = HashMap::new();

    if let Some(ref models) = profile.models {
        let pairs: [(&str, &Option<String>); 5] = [
            ("ANTHROPIC_MODEL", &models.default),
            ("ANTHROPIC_SMALL_FAST_MODEL", &models.small_fast),
            ("ANTHROPIC_DEFAULT_HAIKU_MODEL", &models.default_haiku),
            ("ANTHROPIC_DEFAULT_SONNET_MODEL", &models.default_sonnet),
            ("ANTHROPIC_DEFAULT_OPUS_MODEL", &models.default_opus),
        ];
        for (key, val) in pairs {
            if let Some(ref v) = val {
                env_map.insert(key.to_string(), v.clone());
            }
        }
    }

    if let Some(ref provider) = profile.provider {
        env_map.insert("ANTHROPIC_BASE_URL".into(), provider.base_url.to_string());
        if reveal {
            let token = env::var(&provider.env_key).unwrap_or_else(|_| {
                fatal(&format!(
                    "environment variable {} is not set",
                    provider.env_key
                ));
            });
            env_map.insert("ANTHROPIC_AUTH_TOKEN".into(), token);
            env_map.insert("ANTHROPIC_API_KEY".into(), String::new());
        } else {
            env_map.insert("ANTHROPIC_AUTH_TOKEN".into(), format!("${}", provider.env_key));
            env_map.insert("ANTHROPIC_API_KEY".into(), "(cleared)".into());
        }
    }

    env_map
}

// re-export fatal so config module can use it
fn fatal(msg: &str) -> ! {
    eprintln!("\x1b[31merror\x1b[0m: {msg}");
    std::process::exit(1);
}
