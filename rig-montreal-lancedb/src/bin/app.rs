use std::{env, str::FromStr};

use lambda_runtime::{run, service_fn, tracing::Level, Error, LambdaEvent};
use lancedb::Connection;
use rig::{
    completion::Prompt,
    providers::{self, cohere::EMBED_MULTILINGUAL_LIGHT_V3},
};
use rig_lancedb::{LanceDbVectorStore, SearchParams};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Event {
    prompt: String,
}

#[derive(Serialize)]
struct AgentResponse {
    response: String,
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
    let cohere_client = providers::cohere::Client::new(
        &env::var("COHERE_API_KEY").expect("COHERE_API_KEY not set"),
    );

    // Initialize LanceDb client on EFS mount target
    // Use `/mnt/efs` if data is stored on EFS
    // Use `/tmp` if data is stored on local disk in lambda
    let db = lancedb::connect("data/lancedb").execute().await?;

    run(service_fn(|request: LambdaEvent<Event>| {
        handler(request, &cohere_client, &db)
    }))
    .await
}

async fn handler(
    request: LambdaEvent<Event>,
    cohere_client: &providers::cohere::Client,
    db: &Connection,
) -> Result<AgentResponse, Error> {
    let model = cohere_client.embedding_model(EMBED_MULTILINGUAL_LIGHT_V3, "search_query");

    let table = db.open_table("montreal_open_data").execute().await?;

    // Define search_params params that will be used by the vector store to perform the vector search.
    let search_params = SearchParams::default().distance_type(lancedb::DistanceType::Cosine);
    let index = LanceDbVectorStore::new(table, model, "id", search_params).await?;

    // Create agent with a single context prompt
    let spotify_agent = cohere_client
        .agent("command-r-plus-04-2024")
        .dynamic_context(1, index)
        .build();

    // Prompt the agent and print the response
    let response = spotify_agent.prompt(&request.payload.prompt).await?;

    Ok(AgentResponse { response })
}
