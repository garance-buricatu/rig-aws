use chrono::{DateTime, Utc};
use futures::{stream, FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};

const SPOTIFY_API: &str = "https://api.spotify.com/v1";
const PAGE_SIZE: usize = 50;

pub struct SpotifyClient {
    client: reqwest::Client,
}

impl SpotifyClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn get_all_albums(&self, bearer_token: String) -> impl Stream<Item = AlbumResponse> + '_ {
        let url = format!("{SPOTIFY_API}/me/albums");

        stream::unfold(0, move |offset| {
            self.client
                .get(&url)
                .header(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", bearer_token),
                )
                .query(&[("offset", offset), ("limit", PAGE_SIZE)])
                .send()
                .and_then(|response| response.json::<AlbumResponse>())
                .map(move |album_response| match album_response {
                    Ok(response) => {
                        let results = response.items.clone();

                        if results.len() == 0 {
                            None
                        } else {
                            Some((response, results.len() + offset))
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error fetching open data for offset: {offset} and row: {PAGE_SIZE}. {:?}",
                            e
                        );
                        None
                    }
                })
        })
    }

    pub async fn get_all_artists(
        &self,
        bearer_token: &str,
        ids: Vec<String>,
    ) -> reqwest::Result<impl Iterator<Item = ArtistItem>> {
        let artist_response = stream::iter(ids)
            .chunks(50)
            .map(|id_chunk| async move {
                Ok::<_, reqwest::Error>(
                    self.client
                        .get(&format!("{SPOTIFY_API}/artists"))
                        .header(
                            reqwest::header::AUTHORIZATION,
                            format!("Bearer {}", bearer_token),
                        )
                        .query(&[("ids", id_chunk.join(","))])
                        .send()
                        .await?
                        .json::<ArtistResponse>()
                        .await?,
                )
            })
            .buffer_unordered(10)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(artist_response
            .into_iter()
            .map(|response| response.artists)
            .flatten())
    }
}

#[derive(Deserialize)]
pub struct AlbumResponse {
    pub items: Vec<AlbumItem>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AlbumItem {
    #[serde(rename = "added_at")]
    pub added_to_my_library: DateTime<Utc>,
    pub album: Album,
}

impl AlbumItem {
    pub fn tracks(&self) -> serde_json::Result<Vec<(String, serde_json::Value, Vec<String>)>> {
        self.album.tracks.items.iter().map(|track| {
            let track_value = serde_json::json!({
                "added_to_my_library": self.added_to_my_library,
                "track": serde_json::to_string(track)?,
                "release_date": self.album.release_date,
                "album": self.album.name,
                "artists": self.album.artists.iter().map(|artist| artist.name.clone()).collect::<Vec<_>>(),
            });
            Ok((track.id.clone(), track_value.clone(), vec![serde_json::to_string(&track_value)?]))
        }).collect::<Result<Vec<_>,_>>()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: u32,
    pub id: String,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub artists: Vec<Artist>,
    #[serde(skip_serializing)]
    pub tracks: Tracks,
    pub genres: Vec<String>,
    pub label: String,
    pub popularity: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Tracks {
    pub total: u32,
    pub items: Vec<AlbumTrack>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AlbumTrack {
    pub id: String,
    pub name: String,
    pub duration_ms: u32,
    pub explicit: bool,
    pub track_number: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ArtistResponse {
    pub artists: Vec<ArtistItem>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ArtistItem {
    pub id: String,
    pub name: String,
    pub genres: Vec<String>,
    pub popularity: u32,
    #[serde(rename = "type")]
    pub item_type: String,
}
