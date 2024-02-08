use colored::Colorize;
use serde::{Deserialize, Serialize};

use std::env;
use std::io::{self, Write};
use std::process::exit;

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct Messages {
    messages: Vec<Message>,
}

impl Messages {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct APIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    index: u64,
    finish_reason: String,
    message: Message,
}

#[allow(clippy::struct_field_names)]
#[derive(Serialize, Deserialize)]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

fn in_prompt(
    prompt: &str,
    color: &str,
    input: &mut String,
    history: &mut Messages,
) -> io::Result<()> {
    print!("{}> ", prompt.color(color));
    io::stdout().flush()?;

    input.clear();
    io::stdin().read_line(input)?;
    *input = String::from(input.trim_end());

    if input == "quit" || input == "exit" || input == "bye" {
        exit(0);
    }

    let msg = Message {
        role: String::from(prompt),
        content: input.clone(),
    };
    history.messages.push(msg);

    Ok(())
}

fn out_prompt(prompt: &str, color: &str, input: &String) -> io::Result<()> {
    print!("{}> ", prompt.color(color));
    println!("{input}");
    io::stdout().flush()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut input = String::new();
    let mut history = Messages::new();

    let endpoint =
        env::var("AZURE_OPENAI_ENDPOINT").expect("🔥 AZURE_OPENAI_ENDPOINT must be defined 🔥");
    let deployment = env::var("AZURE_OPENAI_DEPLOYMENTID")
        .expect("🔥 AZURE_OPENAI_DEPLOYMENTID must be defined 🔥");
    let key = env::var("AZURE_OPENAI_APIKEY").expect("🔥 AZURE_OPENAI_APIKEY must be defined 🔥");

    let url = format!(
        "{endpoint}openai/deployments/{deployment}/chat/completions?api-version=2023-05-15"
    );

    println!("Welcome to azchat v0.1.0. Please set a system personnality");
    in_prompt("system", "cyan", &mut input, &mut history)?;

    loop {
        println!();
        in_prompt("user", "green", &mut input, &mut history)?;

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("api-key", &key)
            .body(serde_json::to_string(&history)?)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                match response.json::<APIResponse>().await {
                    Ok(parsed) => {
                        let response_message = &parsed.choices[0].message;
                        history.messages.push(response_message.clone());

                        out_prompt("assistant", "yellow", &response_message.content)?;
                    }
                    Err(_) => println!("Hm, the response didn't match the JSON model we expected."),
                };
            }

            other => {
                panic!("Uh oh! Something unexpected happened: {other:?}");
            }
        };
    }
}
