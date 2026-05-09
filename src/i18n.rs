use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    pub static ref LANG: RwLock<String> = RwLock::new("en".to_string());
    static ref EN_STRINGS: HashMap<String, String> = toml::from_str(include_str!("../locales/en.toml")).unwrap();
    static ref VI_STRINGS: HashMap<String, String> = toml::from_str(include_str!("../locales/vi.toml")).unwrap();
}

pub fn t(key: &str) -> String {
    let lang = LANG.read().unwrap().clone();
    let strings = if lang == "en" { &*EN_STRINGS } else { &*VI_STRINGS };
    strings.get(key).cloned().unwrap_or_else(|| key.to_string())
}

pub fn t_args(key: &str, arg: &str) -> String {
    t(key).replace("{}", arg)
}

pub fn set_lang(lang: &str) {
    if let Ok(mut l) = LANG.write() {
        *l = lang.to_string();
    }
}

pub fn get_lang() -> String {
    LANG.read().unwrap().clone()
}
