use std::error::Error;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};

pub struct Gpt {
    model: String,
}

impl Gpt {
    pub fn new(model: String, api_key: String) -> Result<Self, Box<dyn Error>> {
        openai::set_key(api_key);
        Ok(Self { model })
    }

    pub async fn query(&self) -> Result<String, Box<dyn Error>> {
        let system = "You are in chat conversation. Please type your message.";
        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                name: None,
                content: system.to_string(),
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                name: Some("Alice".to_string()),
                content: "Hello".to_string(),
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                name: Some("Bob".to_string()),
                content: "Hi".to_string(),
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                name: Some("Alice".to_string()),
                content: "Who knows what is the Best programming language for machine learning?".to_string(),
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                name: Some("Charlie".to_string()),
                content: "I don't know. Maybe Rust".to_string(),
            },
        ];

        let model = &self.model.clone();
        let chat = ChatCompletion::builder(model, messages)
            .create()
            .await??;

        let first = chat.choices.first().ok_or("No choices")?;
        Ok(first.message.content.clone())
    }
}