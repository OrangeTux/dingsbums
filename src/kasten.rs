use crate::errors::AppError;
use crate::zettel::Zettel;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Directed;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::prelude::*;
use std::{collections::HashMap, fs::File};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Kasten {
    index: Graph<Uuid, u8, Directed>,
    zettels: HashMap<Uuid, Zettel>,
}

impl Kasten {
    pub fn new() -> Self {
        Kasten {
            index: Graph::new(),
        }
    }

    /// Recreate a `Kasten` from a file.
    pub fn from_file(path: &str) -> Result<Self, AppError> {
        let file = File::open(&path).map_err(AppError::WriteError)?;
        return Kasten::import(file);
    }

    pub fn import<R: Read>(input: R) -> Result<Self, AppError> {
        serde_json::from_reader(input).map_err(AppError::SerializationError)
    }

    pub fn to_file(&self, path: &str) -> Result<(), AppError> {
        let file = File::create(&path).map_err(AppError::WriteError)?;
        return self.export(file);
    }

    pub fn export<W: Write>(&self, output: W) -> Result<(), AppError> {
        serde_json::to_writer(output, &self)
            .map(|_| ())
            .map_err(AppError::SerializationError)
    }

    /// Add a Zettel to a the Kasten.
    pub fn add_zettel(&mut self, zettel: Zettel, parents: Vec<Uuid>) -> Result<(), AppError> {
        if self.get_node_index(zettel.id).is_ok() {
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

        let child_node = self.index.add_node(zettel.id.clone());

        existing_parents.iter().for_each(|parent_node| {
            self.index.add_edge(*parent_node, child_node, 0);
        });

        Ok(())
    }

    fn get_node_index(&self, id: Uuid) -> Result<NodeIndex<u32>, AppError> {
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
