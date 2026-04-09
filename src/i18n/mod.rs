use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod en;
mod phrases;
mod zh_cn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    En,
    ZhCN,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::ZhCN => "zh-CN",
            Language::En => "en",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::ZhCN => "简体中文",
            Language::En => "English",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let normalized = code.trim().to_lowercase().replace('_', "-");

        if normalized.starts_with("zh") {
            Some(Language::ZhCN)
        } else if normalized.starts_with("en") {
            Some(Language::En)
        } else {
            None
        }
    }

    pub fn all() -> Vec<Language> {
        vec![Language::ZhCN, Language::En]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LanguagePreference {
    #[default]
    System,
    ZhCN,
    En,
}

impl LanguagePreference {
    pub fn resolve(self) -> Language {
        match self {
            LanguagePreference::System => get_system_language(),
            LanguagePreference::ZhCN => Language::ZhCN,
            LanguagePreference::En => Language::En,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LanguagePreference::System => "Follow System",
            LanguagePreference::ZhCN => "简体中文",
            LanguagePreference::En => "English",
        }
    }

    pub fn all() -> [LanguagePreference; 3] {
        [
            LanguagePreference::System,
            LanguagePreference::ZhCN,
            LanguagePreference::En,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct I18n {
    pub lang: Language,
    strings: HashMap<String, String>,
    fallback_strings: HashMap<String, String>,
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        let fallback_strings = en::load();
        let strings = load_strings(lang);
        Self {
            lang,
            strings,
            fallback_strings,
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .or_else(|| self.fallback_strings.get(key).cloned())
            .unwrap_or_else(|| key.to_string())
    }

    pub fn t_args(&self, key: &str, args: &HashMap<&str, &str>) -> String {
        let template = self.t(key);
        let mut result = template;
        for (k, v) in args {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }

    pub fn switch(&mut self, lang: Language) {
        self.lang = lang;
        self.strings = load_strings(lang);
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(get_system_language())
    }
}

fn load_strings(lang: Language) -> HashMap<String, String> {
    match lang {
        Language::ZhCN => zh_cn::load(),
        Language::En => en::load(),
    }
}

pub fn use_i18n() -> Signal<I18n> {
    use_context::<Signal<I18n>>()
}

#[macro_export]
macro_rules! t {
    ($i18n:expr, $key:expr) => {
        $i18n.t($key)
    };
    ($i18n:expr, $key:expr, $($arg_key:ident = $arg_val:expr),* $(,)?) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(stringify!($arg_key), $arg_val as &str);
        )*
        $i18n.t_args($key, &args)
    }};
}

pub fn get_system_language() -> Language {
    sys_locale::get_locale()
        .as_deref()
        .and_then(Language::from_code)
        .or_else(|| {
            std::env::var("LANG")
                .ok()
                .as_deref()
                .and_then(Language::from_code)
        })
        .unwrap_or(Language::En)
}
