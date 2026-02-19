use std::sync::OnceLock;

pub mod engine;
pub mod utils;

static LOG_BUFFER: OnceLock<parking_lot::Mutex<Vec<String>>> = OnceLock::new();

pub fn get_log_buffer() -> &'static parking_lot::Mutex<Vec<String>> {
    LOG_BUFFER.get_or_init(|| parking_lot::Mutex::new(Vec::new()))
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{}", msg);
        $crate::get_log_buffer().lock().push(msg);
    }};
}
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { log!("[WARN] {}", format!($($arg)*)) }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { log!("[ERROR] {}", format!($($arg)*)) }
}
