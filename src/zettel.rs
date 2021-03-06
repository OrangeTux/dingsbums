/// A `Zettel` is a note. `Zettel`s are linked to other `Zettel`s with related content.
/// A series of linked `Zettel`s form a undirected connected finite graph. In that a node
/// corresponds to `Zettel`. And the links between nodes (call edges) are the relations between
/// `Zettel`.
use crate::errors::AppError;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use skim::prelude::Cow;
use skim::SkimItem;
use std::io::prelude::*;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct MetaData {
    pub id: Uuid,
    pub title: String,
    pub creation_date: DateTime<Utc>,
}

impl SkimItem for MetaData {
    fn text(&self) -> Cow<str> {
        let text = format!("{} - {}", self.title, self.id);
        Cow::from(text)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Zettel {
    pub meta_data: MetaData,
    pub body: String,

    #[serde(skip)]
    pub dirty: bool,
}

impl std::fmt::Debug for Zettel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.body.len();
        let body: String = match len {
            0..=10 => self.body.clone(),
            _ => {
                // Unwrap can't fail here because we verified the bounds of `body`.
                let start: &str = &self.body.get(0..5).unwrap();
                let end: &str = &self.body.get(len - 5..len).unwrap();

                format!("{}...{}", start, end)
            }
        };

        f.debug_struct("Zettel")
            .field("uuid", &self.meta_data.id)
            // `get(..)` will never fail here. It's equivalent of doing `get(0..body.len())`.
            .field("body", &body.get(..).unwrap())
            .finish()
    }
}

impl Zettel {
    /// Create a child `Zettel`.
    pub fn new(body: String) -> Self {
        let id = Uuid::new_v4();

        let mut zettel = Zettel {
            meta_data: MetaData {
                id,
                title: "".to_string(),
                creation_date: Utc::now(),
            },
            body: "".to_string(),
            dirty: true,
        };

        zettel.update_body(body);
        zettel
    }

    /// Update body and title of `Zettle`.
    pub fn update_body(&mut self, body: String) {
        let title = if let Some(title) = body.lines().next() {
            title
        } else {
            ""
        };

        self.meta_data.title = title.to_string();
        self.dirty = true;
        self.body = body;
    }

    pub fn import<R: Read>(input: R) -> Result<Self, AppError> {
        serde_json::from_reader(input).map_err(AppError::SerializationError)
    }

    pub fn export<W: Write>(&self, output: W) -> Result<(), AppError> {
        serde_json::to_writer(output, &self)
            .map(|_| ())
            .map_err(AppError::SerializationError)
    }
}
