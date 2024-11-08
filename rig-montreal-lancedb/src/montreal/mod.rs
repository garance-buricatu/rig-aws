use api::OpenDataItem;
use serde::{Serialize, Serializer};
use std::fmt::Write;

pub mod api;

#[derive(Debug, Clone)]
pub struct CategoryMetadata {
    pub id: String,
    pub titre: String,
    pub notes: String,
    pub organisation: String,
    pub territoire: Vec<String>,
    pub groupes: Vec<String>,
    pub tags: Vec<String>,
    pub description_donnees: Vec<String>,
    pub methodologie: String,
}

impl From<OpenDataItem> for CategoryMetadata {
    fn from(value: OpenDataItem) -> Self {
        CategoryMetadata {
            id: value.id,
            titre: value.title,
            tags: value.tags.into_iter().map(|t| t.name).collect(),
            groupes: value.groups.into_iter().map(|g| g.name).collect(),
            organisation: value.organization.name,
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

impl Serialize for CategoryMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut paragraph = String::new();

        // Adding each field as a sentence in the paragraph
        write!(&mut paragraph, "{}. ", self.titre).unwrap();
        write!(&mut paragraph, "{}. ", self.notes).unwrap();
        write!(&mut paragraph, "{}. ", self.description_donnees.join(", ")).unwrap();
        write!(&mut paragraph, "{}.", self.methodologie).unwrap();
        write!(&mut paragraph, "Organisation: {}. ", self.organisation).unwrap();
        write!(
            &mut paragraph,
            "Territoire comprend: {}. ",
            self.territoire.join(", ")
        )
        .unwrap();
        write!(
            &mut paragraph,
            "Groupes comprend: {}. ",
            self.groupes.join(", ")
        )
        .unwrap();
        write!(&mut paragraph, "Tags: {}. ", self.tags.join(", ")).unwrap();

        // Serialize the paragraph as a string
        serializer.serialize_str(&paragraph)
    }
}
