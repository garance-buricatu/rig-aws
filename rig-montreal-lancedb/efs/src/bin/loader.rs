use std::{env, str::FromStr, sync::Arc};

use arrow_array::RecordBatchIterator;
use futures::{StreamExt, TryStreamExt};
use lambda_runtime::{run, service_fn, tracing::Level, Error, LambdaEvent};
use lancedb::{index::vector::IvfPqIndexBuilder, Connection};
use rig::{
    embeddings::{EmbeddingModel, EmbeddingsBuilder},
    providers::{self, openai::{TEXT_EMBEDDING_ADA_002}},
};
use rig_spotify_lancedb::{
    arrow_helper::{as_record_batch, schema},
    spotify::{AlbumResponse, SpotifyClient},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Event {
    bearer_token: String,
}

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
    let db = lancedb::connect("/mnt/efs").execute().await?;

    // Initialize Spotify client
    let spotify_client = SpotifyClient::new();

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
) -> Result<Response, Error> {
    let model = &openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    let bearer_token = &request.payload.bearer_token;

    let embeddings = spotify_client
        .get_all_albums(bearer_token.to_string())
        .then(|AlbumResponse { items }| async move {
            let (ids, added_to_my_library): (Vec<_>, Vec<_>) = items
                .iter()
                .flat_map(|album| album.album.artists.iter().map(|artist| (artist.id.clone(), album.added_to_my_library)))
                .unzip();

            let json_documents = spotify_client
                .get_all_artists(bearer_token, ids)
                .await?
                .zip(added_to_my_library)
                .map(|(artist, added_to_my_library)| {
                    let artist_value = serde_json::json!({
                        "added_to_my_library": added_to_my_library,
                        "item": serde_json::to_string(&artist)?,
                    });

                    Ok::<_, serde_json::Error>(vec![(
                        artist.id.clone(),
                        artist_value.clone(),
                        vec![artist_value.to_string()],
                    )])
                })
                .chain(items.iter().map(|album| {
                    let album_str = serde_json::to_string(&album)?;

                    Ok::<_, serde_json::Error>(vec![(
                        album.album.id.clone(),
                        serde_json::to_value(&album)?,
                        vec![album_str],
                    )])
                }))
                .chain(items.iter().map(|album| album.tracks()))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            println!("Length: {}", items.len());

            Ok::<_, anyhow::Error>(
                EmbeddingsBuilder::new(model.clone())
                    .json_documents(json_documents)
                    .build()
                    .await?,
            )
        })
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let record_batch = as_record_batch(embeddings, model.ndims());

    let table = db
        .create_table(
            "my_library",
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
