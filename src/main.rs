mod models;

use std::io::stdin;
use models::{LlmRequest, LlmResponse, Message};
use reqwest::{blocking::{Client, Response}, header::{HeaderMap, HeaderValue}};

const API_KEY: &'static str = env!("MISTRAL_API_KEY");
// const DATA: &'static str = include_str!("../data/sample.json");

fn main() {
    let mut messages: Vec<Message> = vec![];
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(&("Bearer ".to_string() + API_KEY)).unwrap());
    headers.insert("Content-Type", HeaderValue::from_str("application/json").unwrap());
    headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        messages.push(Message { role: String::from("user"), content: input.clone() });

        let llm_req = LlmRequest::from_messages(&messages);
        let mes = serde_json::to_string(&llm_req).expect("Couldn't serialize request");

        let _ = headers.insert("Content-Length", HeaderValue::from_str(stringify!(len(mes))).unwrap());

        let client = Client::builder().default_headers(headers.clone()).build().unwrap();
        let req = client.post("https://api.mistral.ai/v1/chat/completions").body(mes);

        match req.send() {
            Ok(v) => {
                println!("Sent request.");
                if let Err(e) = handle_received(v, &mut messages) {
                    println!("Unsuccessfully handled response.\n  {:?}", e);
                    let res2 = client.post("https://api.mistral.ai/v1/chat/completions");
                    println!("Attempt to print bytes:\n  {:?}", res2.send().unwrap().bytes());
                }
            }
            Err(e) => {
                println!("Unsuccessful request.\n  {:?}", e);
            }
        }
    }
}

fn handle_received(r: Response, messages: &mut Vec<Message>) -> reqwest::Result<()> {
    let response: LlmResponse = r.json()?;
    for choice in &response.choices {
        println!("{}", choice.message.content);
    }
    messages.extend(response.choices.into_iter().map(|c| c.message.to_message()));
    Ok(())
}
