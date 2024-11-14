use std::{env, str::FromStr, sync::Arc};

use arrow_array::RecordBatchIterator;
use futures::StreamExt;
use lambda_runtime::{run, service_fn, tracing::Level, Error, LambdaEvent};
use lancedb::Connection;
use rig::{
    embeddings::{EmbeddingModel, EmbeddingsBuilder},
    providers::{self, openai::TEXT_EMBEDDING_ADA_002},
};
use rig_montreal_lancedb::{
    arrow_helper::{as_record_batch, schema},
    montreal::{api::MontrealOpenDataClient, CategoryMetadata},
};
use serde::{Deserialize, Serialize};
use tiktoken_rs::CoreBPE;

#[derive(Deserialize)]
struct Event {}

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
    let openai_client = providers::openai::Client::new(
        &env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
    );

    // Initialize LanceDb client oon EFS mount target
    // Use `/mnt/efs` if data is stored on EFS
    // Use `/tmp` if data is stored on local disk in lambda
    // Use S3 uri if data is stored in S3
    let db = lancedb::connect("/mnt/efs").execute().await?;

    // Initialize Spotify client
    let montreal_client = MontrealOpenDataClient::new();

    run(service_fn(|request: LambdaEvent<Event>| {
        handler(request, &openai_client, &db, &montreal_client)
    }))
    .await
}

async fn handler(
    _request: LambdaEvent<Event>,
    openai_client: &providers::openai::Client,
    db: &Connection,
    montreal_client: &MontrealOpenDataClient,
) -> Result<Response, Error> {
    let model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    let core_bpe = &tiktoken_rs::cl100k_base()?;

    let embeddings_builder = montreal_client
        .search_all()
        .fold(
            EmbeddingsBuilder::new(model.clone()),
            |builder, opendata_item| async move {
                tracing::info!("Handling item: {}", opendata_item.id);

                let category = CategoryMetadata::from(opendata_item);

                let chunks = chunk(core_bpe, &category);

                let category_json = serde_json::to_value(&category).unwrap();

                builder.json_document(&category.id, category_json.clone(), chunks)
            },
        )
        .await;

    let embeddings = embeddings_builder.build().await.unwrap();

    tracing::info!("Embeddings successfully created!");

    let record_batch = as_record_batch(embeddings, model.ndims());

    db.create_table(
        "montreal_data",
        RecordBatchIterator::new(vec![record_batch], Arc::new(schema(model.ndims()))),
    )
    .execute()
    .await?;

    Ok(Response { success: true })
}

fn chunk(core_bpe: &CoreBPE, value: &CategoryMetadata) -> Vec<String> {
    let document = serde_json::to_string(value).unwrap();
    let tokens = core_bpe.encode_with_special_tokens(&document);

    let mut chunks = vec![];

    if tokens.len() > 8191 {
        let parts = (tokens.len() as f64 / 8191_f64).ceil();
        let split_size = (document.len() as f64 / parts as f64).ceil() as usize;

        for i in 0..parts as usize {
            let document_chunk = document
                [(i * split_size)..std::cmp::min((i + 1) * split_size, document.len() - 1)]
                .to_string();

            chunks.push(document_chunk);
        }
    } else {
        chunks.push(document);
    }

    chunks
}
