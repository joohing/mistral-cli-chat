use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct LlmResponse {
    id: String,
    object: String,
    created: u32,
    model: String,
    pub choices: Vec<LlmChoice>,
    usage: LlmUsage,
}

#[derive(Deserialize, Debug)]
pub struct LlmChoice {
    index: usize,
    pub message: LlmMessage,
    finish_reason: String,
}

#[derive(Deserialize, Debug)]
struct LlmUsage {
    prompt_tokens: u32,
    total_tokens: u32,
    completion_tokens: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LlmMessage {
    role: String,
    tool_calls: Option<Vec<u8>>,
    pub content: String,
}

impl LlmMessage {
    pub fn to_message(self) -> Message {
        Message {
            role: self.role,
            content: self.content,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct LlmRequest {
    model: LlmModel,
    messages: Vec<Message>,
}

impl LlmRequest {
    pub fn from_str(input: &str) -> Self {
        LlmRequest {
            model: LlmModel::MistralLargeLatest,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: input.to_string(),
                }
            ],
        }
    }

    pub fn from_messages(messages: &Vec<Message>) -> Self {
        LlmRequest {
            model: LlmModel::MistralLargeLatest,
            messages: messages.to_vec(),
        }
    }
}

#[derive(Serialize, Debug)]
pub enum LlmModel {
    #[serde(rename = "mistral-large-latest")]
    MistralLargeLatest,
}

impl LlmModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmModel::MistralLargeLatest => {
                "mistral-large-latest"
            }
        }
    }
}
