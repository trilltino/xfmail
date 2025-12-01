use crate::shared::Message;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ChatState {
    pub messages: Vec<Message>,
    pub version_history: HashMap<String, Vec<String>>,
    pub current_version: Option<String>,
    version_counter: u64,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            version_history: HashMap::new(),
            current_version: None,
            version_counter: 0,
        }
    }

    pub fn add_message(&mut self, message: Message, parents: Option<Vec<String>>) -> String {
        self.version_counter += 1;
        let version_id = format!("v{}", self.version_counter);
        
        self.messages.push(message);
        
        let parent_versions = parents.unwrap_or_default();
        self.version_history.insert(version_id.clone(), parent_versions);
        self.current_version = Some(version_id.clone());
        
        version_id
    }

    pub fn get_messages_since(&self, _parent: Option<&String>) -> Vec<Message> {
        self.messages.clone()
    }
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}
