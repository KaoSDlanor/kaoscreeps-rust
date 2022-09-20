use js_sys::JsString;
use web_sys::console;

pub fn colorize<T, C>(text: T, color: C) -> String
where
    T: AsRef<str>,
    C: AsRef<str>,
{
    format!(
        r#"<span style="color:{color}">{text}</span>"#,
        color = color.as_ref(),
        text = text.as_ref()
    )
}

pub fn log<S: AsRef<str>>(str: S) {
    #[allow(unused_unsafe)]
    unsafe {
        console::log_1(&JsString::from(str.as_ref()));
    }
}

pub fn info<S: AsRef<str>>(str: S) {
    log(format!("[INFO] {}", str.as_ref()))
}

pub fn warn<S: AsRef<str>>(str: S) {
    log(format!("[{}] {}", colorize("WARN", "orange"), str.as_ref()))
}

pub fn error<S: AsRef<str>>(str: S) {
    log(format!("[{}] {}", colorize("ERROR", "red"), str.as_ref()))
}

pub fn debug<S: AsRef<str>>(str: S) {
    log(format!(
        "[{}] {}",
        colorize("DEBUG", "dodgerBlue"),
        str.as_ref()
    ))
}
