use api::OpenDataItem;
use serde::Serialize;

pub mod api;

#[derive(Debug, Serialize, Clone)]
pub struct CategoryMetadata {
    pub id: String,
    pub titre: String,
    pub notes: String,
    pub organisation: Organization,
    pub territoire: Vec<String>,
    pub groupes: Vec<Group>,
    pub tags: Vec<Tag>,
    pub description_donnees: Vec<String>,
    pub methodologie: String,
}

impl From<OpenDataItem> for CategoryMetadata {
    fn from(value: OpenDataItem) -> Self {
        CategoryMetadata {
            id: value.id,
            titre: value.title,
            tags: value.tags.into_iter().map(|t| t.into()).collect(),
            groupes: value.groups.into_iter().map(|g| g.into()).collect(),
            organisation: value.organization.into(),
            notes: value.notes,
            territoire: value.territoire,
            description_donnees: value
                .resources
                .into_iter()
                .filter_map(|r| r.description)
                .collect::<Vec<_>>(),
            methodologie: value.methodologie,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Organization {
    pub titre: String,
    pub description: Option<String>,
}

impl From<api::Organization> for Organization {
    fn from(value: api::Organization) -> Self {
        Organization {
            titre: value.title,
            description: value.description,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Group {
    pub description: Option<String>,
    pub titre: String,
}

impl From<api::Group> for Group {
    fn from(value: api::Group) -> Self {
        Group {
            description: value.description,
            titre: value.title,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Tag {
    pub nom: String,
}

impl From<api::Tag> for Tag {
    fn from(value: api::Tag) -> Self {
        Tag { nom: value.name }
    }
}
