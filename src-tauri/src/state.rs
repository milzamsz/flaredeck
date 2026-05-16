use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use tokio::process::Child;

#[derive(Default)]
pub struct RuntimeState {
    pub children: Mutex<HashMap<String, Child>>,
    pub recent_failures: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self::default()
    }
}
