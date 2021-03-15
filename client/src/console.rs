macro_rules! console_log {
    () => {
        ::web_sys::console::log_0()
    };
    ($($arg:tt)*) => {
        ::web_sys::console::log_1(&::wasm_bindgen::JsValue::from(format!($($arg)*)))
    };
}

macro_rules! console_warn {
    () => {
        ::web_sys::console::warn_0()
    };
    ($($arg:tt)*) => {
        ::web_sys::console::warn_1(&::wasm_bindgen::JsValue::from(format!($($arg)*)))
    };
}

macro_rules! console_error {
    () => {
        ::web_sys::console::error_0()
    };
    ($($arg:tt)*) => {
        ::web_sys::console::error_1(&::wasm_bindgen::JsValue::from(format!($($arg)*)))
    };
}
