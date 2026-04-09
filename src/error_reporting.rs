use crate::config::ConfigStorage;
use crate::error::AppError;
use crate::i18n::I18n;
use std::panic::{self, PanicHookInfo};
use std::path::PathBuf;
use tracing::error;

pub struct ErrorReporter {
    log_dir: PathBuf,
}

impl ErrorReporter {
    pub fn init() -> Self {
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rust-redis-desktop")
            .join("logs");

        let _ = std::fs::create_dir_all(&log_dir);

        Self { log_dir }
    }

    pub fn report_fatal_error(error: &AppError) -> ! {
        let error_msg = format!("{}", error);
        let detailed_msg = format!("{:#?}", error);

        eprintln!("\n========================================");
        eprintln!("FATAL ERROR: {}", error_msg);
        eprintln!("========================================\n");
        eprintln!("Details:\n{}\n", detailed_msg);

        if let Some(log_path) = Self::write_error_log(&error_msg, &detailed_msg) {
            eprintln!("Error log saved to: {:?}\n", log_path);
        }

        Self::show_error_dialog(&error_msg);

        std::process::exit(1);
    }

    pub fn report_non_fatal_error(context: &str, error: &dyn std::error::Error) {
        error!("Non-fatal error in {}: {}", context, error);
        eprintln!("[WARN] {} failed: {}", context, error);
    }

    pub fn install_panic_hook() {
        let default_hook = panic::take_hook();

        panic::set_hook(Box::new(move |panic_info| {
            default_hook(panic_info);
            Self::report_panic(panic_info);
        }));
    }

    pub fn report_panic(panic_info: &PanicHookInfo<'_>) {
        let details = Self::format_panic_details(panic_info);
        let summary = "Unexpected panic during application startup or runtime";

        eprintln!("\n========================================");
        eprintln!("PANIC: {}", summary);
        eprintln!("========================================\n");
        eprintln!("Details:\n{}\n", details);

        if let Some(log_path) = Self::write_error_log(summary, &details) {
            eprintln!("Error log saved to: {:?}\n", log_path);
        }

        Self::show_error_dialog(&format!("{summary}\n\n{details}"));
    }

    fn write_error_log(summary: &str, details: &str) -> Option<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let log_file = dirs::config_dir()?
            .join("rust-redis-desktop")
            .join("logs")
            .join(format!("error_{}.log", timestamp));

        let content = format!(
            "Redis Desktop - Fatal Error Log\n\
             Generated: {}\n\
             \n\
             Error Summary:\n{}\n\
             \n\
             Full Details:\n{}\n\
             \n\
             Backtrace:\n{:?}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            summary,
            details,
            std::backtrace::Backtrace::capture()
        );

        std::fs::write(&log_file, content).ok()?;
        Some(log_file)
    }

    fn format_panic_details(panic_info: &PanicHookInfo<'_>) -> String {
        let payload = if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            (*message).to_string()
        } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            message.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        let location = panic_info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "unknown location".to_string());

        format!("Message: {payload}\nLocation: {location}")
    }

    fn show_error_dialog(message: &str) {
        let settings = ConfigStorage::new()
            .ok()
            .and_then(|storage| storage.load_settings().ok())
            .unwrap_or_default();
        let i18n = I18n::new(settings.language_preference.resolve());
        let _ = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title(&format!("Redis Desktop - {}", i18n.t("Startup Error")))
            .set_description(message)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
}

#[macro_export]
macro_rules! fatal_error {
    ($error:expr) => {
        $crate::error_reporting::ErrorReporter::report_fatal_error(&$error)
    };
}

#[macro_export]
macro_rules! non_fatal_error {
    ($context:expr, $error:expr) => {
        $crate::error_reporting::ErrorReporter::report_non_fatal_error($context, &$error)
    };
}
