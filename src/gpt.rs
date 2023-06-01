use std::collections::VecDeque;
use std::error::Error;
use serde::Serialize;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use teloxide::types::UserId;
use crate::chat_data::ChatMember;

pub struct Gpt {
    model: String,
    prompt: String,
    messages_capacity: usize,
    messages: VecDeque<ChatMessage>,
}

impl Gpt {
    pub fn new(api_key: String, model: String, prompt: String, capacity: usize) -> Result<Self, Box<dyn Error>> {
        openai::set_key(api_key);
        let messages = VecDeque::with_capacity(capacity * 2);
        Ok(Self {
            model,
            prompt,
            messages_capacity: capacity,
            messages,
        })
    }

    pub async fn query(&mut self, history: Vec<ChatMessage>) -> Result<Option<String>, Box<dyn Error>> {
        for message in history {
            self.messages.push_back(message);
            if self.messages.len() > self.messages_capacity {
                self.messages.pop_front();
            }
        }

        if self.messages.len() < self.messages_capacity {
            return Ok(None);
        }

        let mut messages = Vec::with_capacity(self.messages_capacity + 1);
        messages.push(ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            name: None,
            content: self.prompt.clone(),
        });

        for message in self.messages.iter() {
            let content = serde_json::to_string(&message.text)?;
            messages.push(ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                name: message.user.name.clone(),
                content,
            });
        }

        let model = &self.model.clone();
        let chat = ChatCompletion::builder(model, messages).create().await??;

        let first = chat.choices.first().ok_or("No choices")?;
        self.messages.clear();

        Ok(Some(first.message.content.clone()))
    }
}



pub struct ChatMessage {
    pub user: ChatMember,
    pub text: ChatMessageJson,
}

impl From<&teloxide::types::Message> for ChatMessage {
    fn from(message: &teloxide::types::Message) -> Self {

        let name = message.from().map(|user| user.id.to_string());

        Self {
            user: ChatMember {
                id: message.from().map(|user| user.id).unwrap_or(UserId(0)),
                name,
            },
            text: message.into(),
        }
    }
}

#[derive(Serialize)]
pub struct ChatMessageJson { //TODO: Make it private
    user_name: String,
    content: String,
}

impl From<&teloxide::types::Message> for ChatMessageJson {
    fn from(message: &teloxide::types::Message) -> Self {

        let user_name = match message.from() {
            Some(user) => user.full_name(), //TODO: be more smart here if full name is empty
            None => "".to_string(), //TODO: log error?
        };

        let content = match message.text() {
            Some(text) => text.to_string(),
            None => "".to_string(), //TODO: log error?
        };

        Self {
            user_name,
            content,
        }
    }
}