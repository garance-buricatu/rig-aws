use std::{env, str::FromStr};

use rig::{completion::Prompt, providers};
use lambda_runtime::{run, service_fn, tracing::Level, Error, LambdaEvent};
use serde::Deserialize;

#[derive(Deserialize)]
struct Event {
    prompt: String
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let env_log_level = std::env::var("LOG_LEVEL").unwrap_or("info".to_string());

    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(&env_log_level).unwrap())
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Initialize the OpenAI client
    let openai_client = providers::openai::Client::new(
        &env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
    );

    run(service_fn(|request: LambdaEvent<Event>| {
        handler(request, &openai_client)
    }))
    .await
}

async fn handler(request: LambdaEvent<Event>, openai_client: &providers::openai::Client) -> Result<(), Error> {
    // Create agent with a single context prompt
    let comedian_agent = openai_client
        .agent("gpt-4o")
        .preamble("You are a comedian here to entertain the user using humour and jokes.")
        .build();

    // Prompt the agent and print the response
    let response = comedian_agent.prompt(&request.payload.prompt).await?;
    println!("{}", response);

    Ok(())
}
