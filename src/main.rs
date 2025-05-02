mod models;

use crate::models::Flags;
use models::{LlmRequest, LlmResponse, Message};
use reqwest::{
    blocking::{Client, Response},
    header::{HeaderMap, HeaderValue},
};
use std::{
    fs::File,
    io::{stdin, Write, Read},
    path::Path,
};

const API_KEY: &'static str = env!("MISTRAL_API_KEY");
const COMPLETIONS_URL: &'static str = "https://api.mistral.ai/v1/chat/completions";

/// Main loop that makes headers, http client, listens for input on stdin, etc.
fn main() {
    let flags = Flags::from(std::env::args().collect());

    // ---- LOAD HISTORY ----
    let history_path = String::from(env!("HOME").to_string() + "/.mistral_cli_chat_history");
    let mut messages: Vec<Message> = vec![];

    if Path::new(&history_path).exists() {
        let history = std::fs::read_to_string(&history_path).unwrap();
        messages = serde_json::from_str(&history).unwrap();
    } else {
        let mut file = File::create(&history_path).unwrap();
        file.write_all(b"[]").unwrap();
    }

    // ---- HEADERS ----
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&("Bearer ".to_string() + API_KEY)).unwrap(),
    );
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("application/json").unwrap(),
    );
    headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

    let mut client = Client::builder()
        .default_headers(headers.clone())
        .build()
        .unwrap();

    // ---- MAIN LOOP ----
    if flags.oneshot {
        read_input_and_send_req(&mut client, &mut headers, &mut messages, &flags);
    }
    else {
        loop {
            read_input_and_send_req(&mut client, &mut headers, &mut messages, &flags);
        }
    }

    std::fs::write(history_path, serde_json::to_string(&messages).unwrap()).unwrap();
}

fn read_input_and_send_req(client: &mut Client, headers: &mut HeaderMap, messages: &mut Vec<Message>, flags: &Flags) {
        let mut input = String::new();
        if flags.oneshot {
            stdin().read_to_string(&mut input).expect("Failed to read all lines into input");
        }
        else {
            stdin().read_line(&mut input).expect("Failed to read line");
        }

        if input == "quit\n".to_string() {
            println!("Writing history to disk and quitting...");
            return;
        }

        messages.push(Message {
            role: String::from("user"),
            content: input.clone(),
        });

        let llm_req = LlmRequest::from_messages(&messages);
        let mes = serde_json::to_string(&llm_req).expect("Couldn't serialize request");

        let _ = headers.insert(
            "Content-Length",
            HeaderValue::from_str(stringify!(len(mes))).unwrap(),
        );

        let req = client.post(COMPLETIONS_URL).body(mes);

        match req.send() {
            Ok(v) => match handle_received(v) {
                Err(e) => {
                    println!("Unsuccessfully handled response.\n  {:?}", e);
                    let res2 = client.post(COMPLETIONS_URL);
                    println!(
                        "Attempt to print bytes:\n  {:?}",
                        res2.send().unwrap().bytes()
                    );
                }
                Ok(new_messages) => messages.extend(new_messages.clone()),
            },
            Err(e) => {
                println!("Unsuccessful request.\n  {:?}", e);
            }
        }
}

/// Upon receiving a response via HTTP, this method is used to parse it into the models
/// specified in src/models.rs and print it to the terminal.
fn handle_received(r: Response) -> reqwest::Result<Vec<Message>> {
    let response: LlmResponse = r.json()?;
    for choice in &response.choices {
        println!("{}", choice.message.content);
    }
    let new_messages: Vec<Message> = response
        .choices
        .into_iter()
        .map(|c| c.message.to_message())
        .collect();
    Ok(new_messages)
}
