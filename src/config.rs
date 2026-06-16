use std::collections::HashMap;
use std::env;

use serde::{Deserialize, Serialize};
use url::Url;

// ── types ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Clone)]
pub struct Provider {
    pub base_url: Url,
    pub env_key: String,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Models {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_haiku: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_sonnet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_opus: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Models>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,
}

#[derive(Deserialize, Serialize)]
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

// ── save ───────────────────────────────────────────────────────────────────

pub fn save_config(config: &Config) {
    let cpath = config_path();
    let content = toml::to_string_pretty(config)
        .unwrap_or_else(|e| fatal(&format!("failed to serialize config: {e}")));
    std::fs::write(&cpath, content)
        .unwrap_or_else(|e| fatal(&format!("cannot write {cpath}: {e}")));
}

// ── field definitions ──────────────────────────────────────────────────────

pub struct FieldDef {
    pub label: &'static str,
    pub section: &'static str,
    pub get: fn(&Profile) -> Option<String>,
    pub set: fn(&mut Profile, String),
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

// Profile section
fn get_extends(p: &Profile) -> Option<String> { p.extends.clone() }
fn set_extends(p: &mut Profile, v: String) { p.extends = none_if_empty(v); }

fn get_description(p: &Profile) -> Option<String> { p.description.clone() }
fn set_description(p: &mut Profile, v: String) { p.description = none_if_empty(v); }

// Models section
fn get_default(p: &Profile) -> Option<String> { p.models.as_ref()?.default.clone() }
fn set_default(p: &mut Profile, v: String) { p.models.get_or_insert_default().default = none_if_empty(v); }

fn get_small_fast(p: &Profile) -> Option<String> { p.models.as_ref()?.small_fast.clone() }
fn set_small_fast(p: &mut Profile, v: String) { p.models.get_or_insert_default().small_fast = none_if_empty(v); }

fn get_default_haiku(p: &Profile) -> Option<String> { p.models.as_ref()?.default_haiku.clone() }
fn set_default_haiku(p: &mut Profile, v: String) { p.models.get_or_insert_default().default_haiku = none_if_empty(v); }

fn get_default_sonnet(p: &Profile) -> Option<String> { p.models.as_ref()?.default_sonnet.clone() }
fn set_default_sonnet(p: &mut Profile, v: String) { p.models.get_or_insert_default().default_sonnet = none_if_empty(v); }

fn get_default_opus(p: &Profile) -> Option<String> { p.models.as_ref()?.default_opus.clone() }
fn set_default_opus(p: &mut Profile, v: String) { p.models.get_or_insert_default().default_opus = none_if_empty(v); }

// Provider section
fn get_base_url(p: &Profile) -> Option<String> { Some(p.provider.as_ref()?.base_url.to_string()) }
fn set_base_url(p: &mut Profile, v: String) {
    if v.is_empty() { return; }
    if let Ok(url) = Url::parse(&v) {
        let pr = match p.provider { Some(ref mut pr) => pr, None => { p.provider = Some(Provider { base_url: url.clone(), env_key: String::new() }); p.provider.as_mut().unwrap() } };
        pr.base_url = url;
    }
}

fn get_env_key(p: &Profile) -> Option<String> { p.provider.as_ref().map(|pr| pr.env_key.clone()) }
fn set_env_key(p: &mut Profile, v: String) {
    let pr = match p.provider { Some(ref mut pr) => pr, None => { p.provider = Some(Provider { base_url: Url::parse("https://localhost").unwrap(), env_key: String::new() }); p.provider.as_mut().unwrap() } };
    pr.env_key = v;
}

pub const PROFILE_FIELDS: &[FieldDef] = &[
    FieldDef { label: "extends",         section: "Profile",  get: get_extends,         set: set_extends },
    FieldDef { label: "description",     section: "Profile",  get: get_description,     set: set_description },
    FieldDef { label: "default",         section: "Models",   get: get_default,         set: set_default },
    FieldDef { label: "small_fast",      section: "Models",   get: get_small_fast,      set: set_small_fast },
    FieldDef { label: "default_haiku",   section: "Models",   get: get_default_haiku,   set: set_default_haiku },
    FieldDef { label: "default_sonnet",  section: "Models",   get: get_default_sonnet,  set: set_default_sonnet },
    FieldDef { label: "default_opus",    section: "Models",   get: get_default_opus,    set: set_default_opus },
    FieldDef { label: "base_url",        section: "Provider", get: get_base_url,        set: set_base_url },
    FieldDef { label: "env_key",         section: "Provider", get: get_env_key,         set: set_env_key },
];

fn fatal(msg: &str) -> ! {
    eprintln!("\x1b[31merror\x1b[0m: {msg}");
    std::process::exit(1);
}
