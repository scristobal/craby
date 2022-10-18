use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::r8client::Input;

pub struct Requests {
    pub inputs: Mutex<HashMap<String, Input>>,
}

impl Requests {
    pub async fn add(&self, id: String, input: Input) -> Option<Input> {
        let mut inputs = self.inputs.lock().await;
        inputs.insert(id, input)
    }

    pub async fn read(&self, id: String) -> Option<Input> {
        let inputs = &self.inputs.lock().await;
        if let Some(input) = inputs.get(&id) {
            return Some(input.clone());
        } else {
            return None;
        }
    }
}
