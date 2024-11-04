use std::{env, str::FromStr, sync::Arc};

use arrow_array::RecordBatchIterator;
use futures::StreamExt;
use lambda_runtime::{run, service_fn, tracing::Level, Error, LambdaEvent};
use lancedb::{index::vector::IvfPqIndexBuilder, Connection};
use rig::{
    embeddings::{EmbeddingModel, EmbeddingsBuilder},
    providers::{self, openai::TEXT_EMBEDDING_ADA_002},
};
use rig_spotify_lancedb::{
    arrow_helper::{as_record_batch, schema},
    montreal::{api::MontrealOpenDataClient, CategoryMetadata},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Event;

#[derive(Serialize)]
struct Response {
    success: bool,
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
        &env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
    );

    // Initialize LanceDb client oon EFS mount target
    let db = lancedb::connect("/mnt/efs").execute().await?;

    // Initialize Spotify client
    let montreal_client = MontrealOpenDataClient::new();

    run(service_fn(|request: LambdaEvent<Event>| {
        handler(request, &cohere_client, &db, &montreal_client)
    }))
    .await
}

async fn handler(
    _request: LambdaEvent<Event>,
    cohere_client: &providers::cohere::Client,
    db: &Connection,
    montreal_client: &MontrealOpenDataClient,
) -> Result<Response, Error> {
    let model = cohere_client.embedding_model(TEXT_EMBEDDING_ADA_002, "search_document");

    let embeddings_builder = montreal_client
        .search_all()
        .fold(
            EmbeddingsBuilder::new(model.clone()),
            |builder, opendata_item| async move {
                let category = CategoryMetadata::from(opendata_item);

                let category_json = serde_json::to_value(&category).unwrap();

                builder.json_document(
                    &category.id,
                    category_json.clone(),
                    vec![category_json.to_string()],
                )
            },
        )
        .await;

    let embeddings = embeddings_builder.build().await.unwrap();

    let record_batch = as_record_batch(embeddings, model.ndims());

    let table = db
        .create_table(
            "montreal_open_data",
            RecordBatchIterator::new(vec![record_batch], Arc::new(schema(model.ndims()))),
        )
        .execute()
        .await?;

    table
        .create_index(
            &["embedding"],
            lancedb::index::Index::IvfPq(
                IvfPqIndexBuilder::default().distance_type(lancedb::DistanceType::Cosine),
            ),
        )
        .execute()
        .await?;

    Ok(Response { success: true })
}
