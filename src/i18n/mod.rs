use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod en;
mod zh_cn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    ZhCN,
    En,
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
        match code.to_lowercase().as_str() {
            "zh-cn" | "zh_cn" | "zh" => Some(Language::ZhCN),
            "en" | "en-us" | "en_us" => Some(Language::En),
            _ => None,
        }
    }

    pub fn all() -> Vec<Language> {
        vec![Language::ZhCN, Language::En]
    }
}

#[derive(Debug, Clone)]
pub struct I18n {
    pub lang: Language,
    strings: HashMap<String, String>,
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        let strings = match lang {
            Language::ZhCN => zh_cn::load(),
            Language::En => en::load(),
        };
        Self { lang, strings }
    }

    pub fn t(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
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
        self.strings = match lang {
            Language::ZhCN => zh_cn::load(),
            Language::En => en::load(),
        };
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::default())
    }
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
    std::env::var("LANG")
        .ok()
        .and_then(|lang| {
            let lang_lower = lang.to_lowercase();
            if lang_lower.starts_with("zh") {
                Some(Language::ZhCN)
            } else if lang_lower.starts_with("en") {
                Some(Language::En)
            } else {
                None
            }
        })
        .unwrap_or_default()
}
