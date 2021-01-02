use crate::errors::AppError;
use crate::zettel::{MetaData, Zettel};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Directed;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::prelude::*;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Kasten {
    // A directed graph that contains the relations between `Zettel`s.
    index: Graph<Uuid, u8, Directed>,

    pub meta_data: HashMap<Uuid, MetaData>,

    #[serde(skip)]
    pub zettels: HashMap<Uuid, Zettel>,
}

impl Kasten {
    pub fn new() -> Self {
        Kasten {
            index: Graph::new(),
            meta_data: HashMap::new(),
            zettels: HashMap::new(),
        }
    }

    /// Create `Kasten` from reader.
    pub fn import<R: Read>(input: R) -> Result<Self, AppError> {
        serde_json::from_reader(input).map_err(AppError::SerializationError)
    }

    pub fn export<W: Write>(&self, output: W) -> Result<(), AppError> {
        serde_json::to_writer(output, &self)
            .map(|_| ())
            .map_err(AppError::SerializationError)
    }

    /// Add a `Zettel` to a `Kasten`.
    /// A `Zettel` can have 0 or more parents.
    pub fn add_zettel(&mut self, zettel: Zettel, parents: Vec<Uuid>) -> Result<(), AppError> {
        if self.get_node_index(zettel.meta_data.id).is_ok() {
            return Err(AppError::ZettelExistsError);
        };

        let mut existing_parents: Vec<NodeIndex<u32>> = vec![];
        let mut non_existing_parents: Vec<Uuid> = vec![];
        parents
            .iter()
            .for_each(|id| match self.get_node_index(id.clone()) {
                Ok(index) => existing_parents.push(index),
                Err(_) => non_existing_parents.push(id.clone()),
            });

        if non_existing_parents.len() > 0 {
            return Err(AppError::ZettelDoesntExistsError);
        }

        let child_node = self.index.add_node(zettel.meta_data.id.clone());

        existing_parents.iter().for_each(|parent_node| {
            self.index.add_edge(*parent_node, child_node, 0);
        });

        let meta_data = zettel.meta_data.clone();

        self.zettels.insert(meta_data.id, zettel);
        self.meta_data.insert(meta_data.id, meta_data);

        Ok(())
    }

    pub fn update_zettel(&mut self, zettel: Zettel) {
        let meta_data = zettel.meta_data.clone();

        self.zettels.insert(meta_data.id, zettel);
        self.meta_data.insert(meta_data.id, meta_data);
    }

    pub fn get_zettel(&self, id: Uuid) -> Result<Zettel, AppError> {
        Ok(self.zettels[&id].clone())
    }

    pub fn get_node_index(&self, id: Uuid) -> Result<NodeIndex<u32>, AppError> {
        match self.index.node_indices().find(|i| self.index[*i] == id) {
            Some(i) => Ok(i),
            None => Err(AppError::ZettelDoesntExistsError),
        }
    }

    pub fn dot(&self) {
        use petgraph::dot::{Config, Dot};
        println!(
            "{:?}",
            Dot::with_config(&self.index, &[Config::EdgeNoLabel])
        );
    }
}
