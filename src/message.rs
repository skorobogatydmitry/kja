//! messenging to the end-user
//!

use std::sync::Mutex;

static MESSAGE: Mutex<Option<String>> = Mutex::new(None);

/// return current message
pub fn current() -> Option<String> {
    MESSAGE.lock().unwrap().clone()
}

/// replace current message with the new one
pub fn show(m: String) {
    MESSAGE.lock().unwrap().replace(m);
}
