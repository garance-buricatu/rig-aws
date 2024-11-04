use api::OpenDataItem;
use serde::Serialize;

pub mod api;

#[derive(Debug, Serialize, Clone)]
pub struct CategoryMetadata {
    pub id: String,
    pub title: String,
    pub notes: String,
    pub organization: Organization,
    pub territory: Vec<String>,
    pub groups: Vec<Group>,
    pub tags: Vec<Tag>,
    pub dataset_descriptions: Vec<String>,
    pub methodology: String,
}

impl From<OpenDataItem> for CategoryMetadata {
    fn from(value: OpenDataItem) -> Self {
        CategoryMetadata {
            id: value.id,
            title: value.title,
            tags: value.tags.into_iter().map(|t| t.into()).collect(),
            groups: value.groups.into_iter().map(|g| g.into()).collect(),
            organization: value.organization.into(),
            notes: value.notes,
            territory: value.territoire,
            dataset_descriptions: value
                .resources
                .into_iter()
                .filter_map(|r| r.description)
                .collect::<Vec<_>>(),
            methodology: value.methodologie,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Organization {
    pub title: String,
    pub description: Option<String>,
}

impl From<api::Organization> for Organization {
    fn from(value: api::Organization) -> Self {
        Organization {
            title: value.title,
            description: value.description,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Group {
    pub description: Option<String>,
    pub title: String,
}

impl From<api::Group> for Group {
    fn from(value: api::Group) -> Self {
        Group {
            description: value.description,
            title: value.title,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Tag {
    pub name: String,
}

impl From<api::Tag> for Tag {
    fn from(value: api::Tag) -> Self {
        Tag { name: value.name }
    }
}
