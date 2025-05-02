use serde::{Deserialize, Serialize};

/// The "top-level" of the API's response contains these fields.
#[derive(Deserialize, Debug)]
pub struct LlmResponse {
    id: String,
    object: String,
    created: u32,
    model: String,
    pub choices: Vec<LlmChoice>,
    usage: LlmUsage,
}

/// No idea what this means, but contains the actual content of the response inside `message`.
#[derive(Deserialize, Debug)]
pub struct LlmChoice {
    index: usize,
    pub message: LlmMessage,
    finish_reason: String,
}

/// How much the LLM spent on different scheiße.
#[derive(Deserialize, Debug)]
struct LlmUsage {
    prompt_tokens: u32,
    total_tokens: u32,
    completion_tokens: u32,
}

/// Contents of the message, etc.
#[derive(Deserialize, Serialize, Debug)]
pub struct LlmMessage {
    role: String,
    tool_calls: Option<Vec<u8>>,
    pub content: String,
}

impl LlmMessage {
    /// Convenience method that just removes the `tool_calls` field so that you can push it to a
    /// vec of `Message`.
    pub fn to_message(self) -> Message {
        Message {
            role: self.role,
            content: self.content,
        }
    }
}

/// Is used for sending a message, as well as storing the message history.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// The "top-level" of the http request body that we send to the Mistral API.
#[derive(Serialize, Debug)]
pub struct LlmRequest {
    model: LlmModel,
    messages: Vec<Message>,
}

impl LlmRequest {
    /// When input is read from stdin
    pub fn from_str(input: &str) -> Self {
        LlmRequest {
            model: LlmModel::MistralLargeLatest,
            messages: vec![Message {
                role: "user".to_string(),
                content: input.to_string(),
            }],
        }
    }

    /// This is used after pushing the latest message to `messages`, and simply wraps the messages
    /// to add info about the model type as well.
    pub fn from_messages(messages: &Vec<Message>) -> Self {
        LlmRequest {
            model: LlmModel::MistralLargeLatest,
            messages: messages.to_vec(),
        }
    }
}

/// Availabe model types from the Mistral API. (Obviously incomplete you dummy)
#[derive(Serialize, Debug)]
pub enum LlmModel {
    #[serde(rename = "mistral-large-latest")]
    MistralLargeLatest,
}

impl LlmModel {
    /// Unused since I just tag the field with a serde rename instead LMAO
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmModel::MistralLargeLatest => "mistral-large-latest",
        }
    }
}

/// Filetypes are set from the flags passed in to the program. They
/// help the user tell the LLM the context they are in.
#[derive(Serialize, Debug, Clone)]
pub enum FileType {
    Rust,
}

#[derive(Debug)]
pub struct Flags {
    pub oneshot: bool,
    pub extension: String,
}

impl Flags {
    pub fn from(args: Vec<String>) -> Flags {
        let mut oneshot = false;
        for argument in args.into_iter().skip(1) {
            match argument.as_str() {
                "-o" => {
                    oneshot = true;
                },
                &_ => {
                    println!("Warning: unknown argument {}", argument);
                }
            }
        }
        Flags {
            oneshot,
            extension: String::from("hello"),
        }
    }
}
