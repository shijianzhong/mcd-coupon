use chrono::Local;

/// Format current time as string
pub fn format_current_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format a log message with timestamp
pub fn format_log_message(message: &str) -> String {
    format!("[{}] {}", format_current_time(), message)
}

/// Truncate string to fit within a certain width
pub fn truncate_string(s: &str, width: usize) -> String {
    if s.len() <= width {
        s.to_string()
    } else {
        format!("{:.width$}...", s, width = width - 3)
    }
}
