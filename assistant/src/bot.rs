use anyhow::bail;
use openai_api_rs::v1::{api::Client, audio::AudioSpeechRequest, chat_completion::{self, ChatCompletionMessage, ChatCompletionRequest}, common::GPT3_5_TURBO};


pub struct OpenAIChatCompletion {
    client: Client,
    messages: Vec<openai_api_rs::v1::chat_completion::ChatCompletionMessage>
}


impl OpenAIChatCompletion {
    pub fn new(api_key: Option<String>) -> anyhow::Result<Self> {
        let api_key = api_key.unwrap_or_else(|| std::env::var("OPENAI_API_KEY").expect("No OPENAI_API_KEY provided"));

        Ok(Self {
            client: Client::new(api_key),
            messages: Vec::with_capacity(128)
        })
    }

    pub fn speak(&mut self, message: &str) -> anyhow::Result<String> {
        let outbound_message = ChatCompletionMessage {
            role: openai_api_rs::v1::chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(message.to_string()),
            name: None
        };
        let mut previous_messages = self.messages.clone();
        previous_messages.push(outbound_message);
        let request = ChatCompletionRequest::new(GPT3_5_TURBO.to_string(), previous_messages);
        let result = self.client.chat_completion(request)?;
        let msg = &result.choices[0].message;
        let Some(content) = &result.choices[0].message.content else {
            bail!("No content found!");
        };
        self.messages.push(
            ChatCompletionMessage {
                role: msg.role.clone(),
                content: chat_completion::Content::Text(msg.content.clone().unwrap_or_default()),
                name: msg.name.clone()
            }
        );
        Ok(content.to_string())
    }
}