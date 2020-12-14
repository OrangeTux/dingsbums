/// A `Zettel` is a note. `Zettel`s are linked to other `Zettel`s with related content.
/// A series of linked `Zettel`s form a undirected connected finite graph. In that a node
/// corresponds to `Zettel`. And the links between nodes (call edges) are the relations between
/// `Zettel`.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Zettel {
    pub id: Uuid,
    pub body: String,
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
            .field("uuid", &self.id)
            // `get(..)` will never fail here. It's equivalent of doing `get(0..body.len())`.
            .field("body", &body.get(..).unwrap())
            .finish()
    }
}

impl Zettel {
    /// Create a child `Zettel`.
    pub fn new(body: String) -> Self {
        let id = Uuid::new_v4();

        Zettel { id, body }
    }
}
