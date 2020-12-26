use crate::errors::AppError;
use crate::zettel::Zettel;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Directed;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::prelude::*;
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    path::Path,
};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Kasten {
    index: Graph<Uuid, u8, Directed>,

    #[serde(skip)]
    zettels: HashMap<Uuid, Zettel>,
}

impl Kasten {
    pub fn new() -> Self {
        Kasten {
            index: Graph::new(),
            zettels: HashMap::new(),
        }
    }

    /// Create a `ZettelKasten` by importing directory at given `path`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let kasten = Kasten::from_dir("~/.zettelkasten").unwrap();
    /// ```
    pub fn from_dir(path: &str) -> Result<Self, AppError> {
        let path = Path::new(&path).join("db");
        let file = File::open(&path).map_err(AppError::WriteError)?;
        return Kasten::import(file);
    }

    fn import<R: Read>(input: R) -> Result<Self, AppError> {
        serde_json::from_reader(input).map_err(AppError::SerializationError)
    }

    /// Export a `ZettelKasten` to a file system at the given `path`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let kasten = Kasten::new();
    /// kasten.to_dir("~/.zettelkasten").unwrap();
    /// ```
    pub fn to_dir(&self, path: &str) -> Result<(), AppError> {
        let dir = Path::new(path);
        if !dir.exists() {
            create_dir_all(&path.clone()).map_err(AppError::WriteError)?;
        }

        let path = Path::new(&path).join("db");
        let file = File::create(&path).map_err(AppError::WriteError)?;
        self.export(file)?;

        for zettel in self.zettels.values() {
            if !zettel.dirty {
                continue;
            };

            let path = Path::new(&dir).join(zettel.id.to_string());
            let file = File::create(&path).map_err(AppError::WriteError)?;
            zettel.export(&file)?
        }

        Ok(())
    }

    fn export<W: Write>(&self, output: W) -> Result<(), AppError> {
        serde_json::to_writer(output, &self)
            .map(|_| ())
            .map_err(AppError::SerializationError)
    }

    /// Add a `Zettel` to a `Kasten`.
    /// A `Zettel` can have 0 or more parents.
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

        self.zettels.insert(zettel.id, zettel);

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
