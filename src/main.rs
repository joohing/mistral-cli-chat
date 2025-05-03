mod models;

use atty::Stream;
use crate::models::Args;
use models::{LlmRequest, LlmResponse, Message};
use reqwest::{
    blocking::{Client, Response},
    header::{HeaderMap, HeaderValue},
};
use std::{
    fs::File,
    io::{stdin, Read, Write},
    path::Path,
};

const API_KEY: &'static str = env!("MISTRAL_API_KEY");
const COMPLETIONS_URL: &'static str = "https://api.mistral.ai/v1/chat/completions";
const STFU_TEXT: &'static str = "You are a CLI tool for use in generation of small snippets of code for direct insertion into the editor. Perceive yourself like a form of rustfmt, except in addition to formatting, you may do other things like generate a function, or take a function and rewrite it.\nIn your following response, DO NOT:\n- Give any syntactically invalid text\n- Give any code except EXACTLY what you are asked for\n- Put markdown markers around code (they are not valid syntax)\n- Write any explanatory text\n\nIf you get some lines of code:\n- ONLY perform the operation requested on the given lines of code.\n- Don't reprint anything except the given lines. Your input is the following:\n";

/// Main loop that makes headers, http client, listens for input on stdin, etc.
fn main() {
    let args = Args::from(std::env::args().collect());
    if args.help {
        print_help_information();
        return;
    }

    // ---- LOAD HISTORY ----
    let history_path = String::from(env!("HOME").to_string() + "/.mistral_cli_chat_history");
    let mut messages: Vec<Message> = vec![];

    if args.context {
        if Path::new(&history_path).exists() {
            let history = std::fs::read_to_string(&history_path).unwrap();
            messages = serde_json::from_str(&history).unwrap();
        } else {
            let mut file = File::create(&history_path).unwrap();
            file.write_all(b"[]").unwrap();
        }
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
    while read_input_and_send_req(&mut client, &mut headers, &mut messages, &args) {}

    // If not in conversation it's probably gonna be pretty annoying that it prints this message lol
    if args.long { println!("Writing history to disk and quitting..."); }

    if args.context { std::fs::write(history_path, serde_json::to_string(&messages).unwrap()).unwrap(); }
}

fn read_input_and_send_req(
    client: &mut Client,
    headers: &mut HeaderMap,
    messages: &mut Vec<Message>,
    args: &Args,
) -> bool {
    let mut input = String::new();

    handle_read_input(&mut input, args);

    if input == "wq\n".to_string() {
        return false;
    }

    let content = format!(
        "{}\n{}\n{}",
        if args.long { "" } else { STFU_TEXT },
        args.llm_input.join(" "),
        input.clone()
    );

    messages.push(Message {
        role: String::from("user"),
        content,
    });

    let llm_req = LlmRequest::from_messages(&messages);
    let mes = serde_json::to_string(&llm_req).expect("Couldn't serialize request");

    let _ = headers.insert(
        "Content-Length",
        HeaderValue::from_str(stringify!(len(mes))).unwrap(),
    );

    let req = client.post(COMPLETIONS_URL).body(mes);

    match req.send() {
        Ok(v) => match handle_received(v, &args) {
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

    if args.long { true } else { false }
}

/// Parses received HTTP response into models and prints to terminal.
fn handle_received(r: Response, args: &Args) -> reqwest::Result<Vec<Message>> {
    let response: LlmResponse = r.json()?;
    for choice in &response.choices {
        let p = if args.long {
            &choice.message.content
        }
        else {
            &choice.message.content.lines()
                .filter(|line| !line.starts_with("```"))
                .collect::<Vec<&str>>()
                .join("\n")
        };

        println!("{}", p);
    }
    Ok(response
        .choices
        .into_iter()
        .map(|c| c.message.to_message())
        .collect())
}

/// Handle all the funky stuff with reading input.
/// - Non-term should always just read all till EOF.
/// - Term should see if input was given when typing the cmd to run the exe.
///     - If so, and extended is enabled, let the user keep typing stuff.
///     - If so, and extended is not enabled, don't read from input.
fn handle_read_input(input: &mut String, args: &Args) {
    if args.extended {
        stdin()
            .read_to_string(input)
            .expect("Failed to read all lines into input");
        return;
    }
    if args.llm_input.len() == 0 {
        stdin().read_line(input).expect("Failed to read line");
    }
}

fn print_help_information() {
    print!(
        r#"
mistral-cli: Sends queries from the terminal. Persistent conversation history saved in `~/.mistral_cli_chat_history`.
Only the latest 100 messages are saved, but can be changed easily in the source code.

Uses different flags to specify the way to treat input.
By default: reads until EOF, gets a response, and exits after saving to history.

Possible flags:
    [-l|--long]         =>      Don't exit after one message.
    [-e|--extended]     =>      Wait for an EOF character ^D before sending. Good for vim.
    [-c|--context]      =>      Enable loading the history.
    [-h|--help]         =>      Print this help message.

Anything not a flag is passed directly to the LLM."#
    );
}
