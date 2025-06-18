use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};

#[derive(Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    input: Vec<Message>,
    text: TextFormat,
    reasoning: Reasoning,
    tools: Vec<Tool>,
    temperature: f32,
    max_output_tokens: u32,
    top_p: f32,
    store: bool,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct TextFormat {
    format: FormatType,
}

#[derive(Serialize, Deserialize)]
struct FormatType {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Serialize, Deserialize)]
struct Reasoning {}

#[derive(Serialize, Deserialize)]
struct Tool {}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    output: Vec<OutputMessage>,
}

#[derive(Debug, Deserialize)]
struct OutputMessage {
    content: Vec<OutputContent>,
}

#[derive(Debug, Deserialize)]
struct OutputContent {
    text: String,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY")?;

    // Ambil input dari terminal
    print!("Masukkan pesan untuk GPT: ");
    io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;
    let user_input = user_input.trim();

    // Bentuk payload
    let payload = OpenAIRequest {
        model: "gpt-4.1".to_string(),
        input: vec![Message {
            role: "user".to_string(),
            content: vec![Content {
                content_type: "input_text".to_string(),
                text: user_input.to_string(),
            }],
        }],
        text: TextFormat {
            format: FormatType {
                format_type: "text".to_string(),
            },
        },
        reasoning: Reasoning {},
        tools: vec![],
        temperature: 1.0,
        max_output_tokens: 2048,
        top_p: 1.0,
        store: true,
    };

    let client = Client::new();
    let res = client
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await?;

    let status = res.status();
    println!("Status: {}", status);
    
    let parsed: OpenAIResponse = res.json().await?;

    if let Some(first_msg) = parsed.output.get(0) {
        if let Some(first_content) = first_msg.content.get(0) {
            println!("GPT says: {}", first_content.text);
        } else {
            println!("No content found in message.");
        }
    } else {
        println!("No message found in output.");
    }


    Ok(())
}
