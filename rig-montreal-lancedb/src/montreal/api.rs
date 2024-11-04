use chrono::Utc;
use futures::{stream, FutureExt, Stream, StreamExt, TryFutureExt};
use serde::Deserialize;

const MONTREAL_OPEN_DATA_API: &str = "https://donnees.montreal.ca/api/3/action";
const PAGE_SIZE: usize = 50;

pub struct MontrealOpenDataClient {
    client: reqwest::Client,
    url: String,
}

impl Default for MontrealOpenDataClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MontrealOpenDataClient {
    pub fn new() -> Self {
        Self::from_url(MONTREAL_OPEN_DATA_API)
    }

    pub fn from_url(url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        }
    }

    pub fn search_all(&self) -> impl Stream<Item = OpenDataItem> + '_ {
        let url = format!("{}/{}", self.url, "package_search");

        stream::unfold(0, move |skip| {
            self.client
                .get(&url)
                .query(&[("start", skip), ("rows", PAGE_SIZE)])
                .send()
                .and_then(|response| response.json::<OpenDataResponse>())
                .map(move |open_data_response| match open_data_response {
                    Ok(response) => {
                        let results = response.result.results;

                        if results.len() == 0 {
                            None
                        } else {
                            tracing::info!("Fetched {} results", results.len());
                            Some((stream::iter(results.clone()), results.len() + skip))
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error fetching open data for skip: {skip} and row: {PAGE_SIZE}. {:?}",
                            e
                        );
                        None
                    }
                })
        })
        .flatten()
    }
}

mod montreal_data_date_format_1 {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6f";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let o: Option<String> = Option::deserialize(deserializer)?;

        match o {
            Some(s) => {
                let dt =
                    NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
                Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenDataResponse {
    pub result: OpenDataResult,
}

#[derive(Debug, Deserialize)]
pub struct OpenDataResult {
    pub count: i32,
    pub results: Vec<OpenDataItem>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenDataItem {
    pub author: String,
    pub author_email: String,
    pub creator_user_id: String,
    pub ext_spatial: String,
    pub extras_flag_donnees_normalise: Option<String>,
    pub id: String,
    pub isopen: bool,
    pub language: String,
    pub license_id: String,
    pub license_title: String,
    pub license_url: String,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    #[serde(default, with = "montreal_data_date_format_1")]
    pub metadata_created: Option<chrono::DateTime<Utc>>,
    #[serde(default, with = "montreal_data_date_format_1")]
    pub metadata_modified: Option<chrono::DateTime<Utc>>,
    pub methodologie: String,
    pub name: String,
    pub notes: String,
    pub num_resources: u32,
    pub num_tags: u32,
    pub organization: Organization,
    pub owner_org: String,
    pub private: bool,
    pub state: String,
    pub temporal: Option<String>,
    pub territoire: Vec<String>,
    pub title: String,
    pub r#type: String,
    pub update_frequency: String,
    pub url: String,
    pub version: Option<String>,
    pub groups: Vec<Group>,
    pub resources: Vec<Resource>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub title: String,
    pub r#type: String,
    pub description: Option<String>,
    pub image_url: String,
    #[serde(default, with = "montreal_data_date_format_1")]
    pub created: Option<chrono::DateTime<Utc>>,
    pub is_organization: bool,
    pub approval_status: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Group {
    pub description: Option<String>,
    pub display_name: String,
    pub id: String,
    pub image_display_url: String,
    pub name: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    pub display_name: String,
    pub id: String,
    pub name: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Resource {
    pub cache_last_updated: Option<String>,
    pub cache_url: Option<String>,
    #[serde(default, with = "montreal_data_date_format_1")]
    pub created: Option<chrono::DateTime<Utc>>,
    pub datastore_active: bool,
    pub datastore_contains_all_records_of_source_file: bool,
    pub description: Option<String>,
    pub format: Option<String>,
    pub hash: String,
    pub id: String,
    #[serde(default, with = "montreal_data_date_format_1")]
    pub last_modified: Option<chrono::DateTime<Utc>>,
    #[serde(default, with = "montreal_data_date_format_1")]
    pub metadata_modified: Option<chrono::DateTime<Utc>>,
    pub mimetype: Option<String>,
    pub mimetype_inner: Option<String>,
    pub name: String,
    pub package_id: String,
    pub position: u32,
    pub relidi_condon_boolee: Option<String>,
    pub relidi_condon_datheu: Option<String>,
    pub relidi_condon_nombre: Option<String>,
    pub relidi_condon_valinc: Option<String>,
    pub relidi_confic_epsg: Option<String>,
    pub relidi_confic_pascom: Option<String>,
    pub relidi_confic_separateur_virgule: Option<String>,
    pub relidi_confic_utf8: Option<String>,
    pub relidi_description_champs: Option<String>,
    pub relidi_ressource_complementaire: Vec<String>,
    pub resource_type: Option<String>,
    pub size: Option<u64>,
    pub state: String,
    pub url: String,
    pub url_type: Option<String>,
}
