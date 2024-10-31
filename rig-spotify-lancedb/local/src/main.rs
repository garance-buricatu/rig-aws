use std::{env, str::FromStr, sync::Arc};

use arrow_array::RecordBatchIterator;
use arrow_helper::{as_record_batch, schema};
use lambda_runtime::{run, service_fn, tracing::Level, Error, LambdaEvent};
use lancedb::{index::vector::IvfPqIndexBuilder, Connection};
use rig::{
    completion::Prompt,
    embeddings::{DocumentEmbeddings, EmbeddingModel},
    providers::{self, openai::TEXT_EMBEDDING_ADA_002},
};
use rig_lancedb::{LanceDbVectorStore, SearchParams};
use serde::{Deserialize, Serialize};
use spotify::SpotifyClient;
mod arrow_helper;
mod spotify;

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
    let openai_client = providers::openai::Client::new(
        &env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
    );

    // Initialize LanceDb client in /tmp folder of lambda (where ephemeral storage is mounted).
    let db = lancedb::connect("/tmp").execute().await?;

    // Initialize Spotify client
    let spotify_client = SpotifyClient::new(
        std::env::var("RAPIDAPI_KEY").expect("RAPIDAPI_KEY not set!")
    );

    run(service_fn(|request: LambdaEvent<Event>| {
        handler(request, &openai_client, &db, &spotify_client)
    }))
    .await
}

async fn handler(
    request: LambdaEvent<Event>,
    openai_client: &providers::openai::Client,
    db: &Connection,
    spotify_client: &SpotifyClient,
) -> Result<AgentResponse, Error> {
    let model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    let embeddings = spotify_client.create_embeddings(&model).await?;

    let table = fill_table(&model, db, embeddings).await?;

    // Define search_params params that will be used by the vector store to perform the vector search.
    let search_params = SearchParams::default();
    let index = LanceDbVectorStore::new(table, model, "id", search_params).await?;

    // Create agent with a single context prompt
    let spotify_agent = openai_client
        .agent("gpt-4o")
        .dynamic_context(1, index)
        .build();

    // Prompt the agent and print the response
    let response = spotify_agent.prompt(&request.payload.prompt).await?;

    Ok(AgentResponse { response })
}

async fn fill_table(
    model: &impl EmbeddingModel,
    db: &Connection,
    embeddings: Vec<DocumentEmbeddings>,
) -> Result<lancedb::Table, lancedb::Error> {
    let record_batch = as_record_batch(embeddings, model.ndims());

    let table = db
        .create_table(
            "artist_overview",
            RecordBatchIterator::new(vec![record_batch], Arc::new(schema(model.ndims()))),
        )
        .execute()
        .await?;

    // See [LanceDB indexing](https://lancedb.github.io/lancedb/concepts/index_ivfpq/#product-quantization) for more information
    table
        .create_index(
            &["embedding"],
            lancedb::index::Index::IvfPq(IvfPqIndexBuilder::default()),
        )
        .execute()
        .await?;

    Ok(table)
}
