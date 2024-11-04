use futures::{StreamExt, TryStreamExt};
use rig::embeddings::{DocumentEmbeddings, EmbeddingModel, EmbeddingsBuilder};
use serde::{Deserialize, Serialize};

pub struct SpotifyClient {
    http_client: reqwest::Client,
    api_key: String,
}

impl SpotifyClient {
    pub fn new(api_key: String) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            http_client,
            api_key
        }
    }

    pub async fn artist_overview(
        &self,
        artist_id: &str,
    ) -> Result<ArtistOverviewResponse, anyhow::Error> {
        let response = self
            .http_client
            .get("https://spotify-scraper.p.rapidapi.com/v1/artist/overview")
            .headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "X-RapidAPI-Host",
                    "spotify-scraper.p.rapidapi.com".parse().unwrap(),
                );
                headers.insert("X-RapidAPI-Key", self.api_key.parse().unwrap());
                headers
            })
            .query(&[("artistId", artist_id)])
            .send()
            .await?
            .text()
            .await?;

        match serde_json::from_str(&response) {
            Ok(overview) => Ok(overview),
            Err(e) => {
                tracing::error!("Failed to parse artist {artist_id} overview response: {response}. {e:?}");
                Err(e.into())
            }
        }
    }

    pub async fn create_embeddings(
        &self,
        model: &impl EmbeddingModel,
    ) -> anyhow::Result<Vec<DocumentEmbeddings>> {
        let artist_ids = vec![
            "7oPftvlwr6VrsViSDV7fJY",
            "1Xyo4u8uXC1ZmMpatF05PJ",
            "3qm84nBOXUEQ2vnTfUTTFC",
            "22bE4uQ6baNwSHPVcDxLCe",
            "74ASZWbe4lXaubB36ztrGX",
            "3PhoLpVuITZKcymswpck5b",
            "6eUKZXaKkcviH0Ku9w2n3V",
            "6vWDO969PvNqNYHIOW5v0m",
            "0oSGxfWSnnOXhD2fKuz2Gy",
            "3fMbdgg4jU18AjLCKBhRSm",
            "72OaDtakiy6yFqkt4TsiFt",
            "5pKCCKE2ajJHZ9KAiaK11H",
            "1dfeR4HaWDbWqFHLkxsg1d"
        ];

        let json_documents = futures::stream::iter(artist_ids)
            .map(|id| async move {
                let ArtistOverviewResponse {
                    id,
                    name,
                    biography,
                    discography,
                    goods,
                } = self.artist_overview(id).await?;

                let concerts = goods
                    .concerts
                    .items
                    .iter()
                    .map(|concert_item| {
                        Ok((
                            format!("concert-{}", concert_item.id),
                            serde_json::json!(name),
                            vec![serde_json::to_string(concert_item)?],
                        ))
                    })
                    .collect::<Result<Vec<_>, serde_json::Error>>()?;

                let albums = discography
                    .albums
                    .items
                    .iter()
                    .map(|album| {
                        Ok((
                            format!("album-{}", album.id),
                            serde_json::json!(name),
                            vec![serde_json::to_string(album)?],
                        ))
                    })
                    .collect::<Result<Vec<_>, serde_json::Error>>()?;

                let singles = discography
                    .singles
                    .items
                    .iter()
                    .map(|single| {
                        Ok((
                            format!("single-{}", single.id),
                            serde_json::json!(name),
                            vec![serde_json::to_string(single)?],
                        ))
                    })
                    .collect::<Result<Vec<_>, serde_json::Error>>()?;

                let mut json_documents = vec![(
                    format!("biography-{}", id),
                    serde_json::json!(name),
                    vec![biography.clone()],
                )];

                json_documents.extend(concerts);
                json_documents.extend(albums);
                json_documents.extend(singles);

                Ok::<_, anyhow::Error>(json_documents)
            })
            // Rate limiting from Rapid API
            .buffer_unordered(1)
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        println!("Len: {}", json_documents.len());

        Ok(EmbeddingsBuilder::new(model.clone())
            .json_documents(json_documents)
            .build()
            .await?)
    }
}

#[derive(Deserialize)]
pub struct ArtistOverviewResponse {
    pub id: String,
    pub name: String,
    pub biography: String,
    pub discography: Discography,
    pub goods: Goods,
}

#[derive(Deserialize)]
pub struct Discography {
    pub albums: Album,
    pub singles: Single
}

#[derive(Deserialize)]
pub struct Single {
    pub items: Vec<DiscographyItem>
}

#[derive(Deserialize)]
pub struct Album {
    pub items: Vec<DiscographyItem>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscographyItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub name: String,
    pub label: String,
    pub track_count: u32,
}

#[derive(Deserialize)]
pub struct Goods {
    pub concerts: Concerts,
}

#[derive(Deserialize)]
pub struct Concerts {
    pub items: Vec<Concert>,
}

#[derive(Deserialize, Serialize)]
pub struct Concert {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub date: String,
    pub venue: String,
    pub location: String,
}
