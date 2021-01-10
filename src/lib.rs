//! # Dingsbums
//!
//! `dingsbums` is a crate implementing the [knowledge storage system][zettelkasten-wiki] called
//! "ZettelKasten" as invented by [Niklas Luhmann][luhman].
//!
//! `dingsbums` consists of collection of notes called [`Zettel`]s. Each note captures an idea or
//! knowledge related to a certain topic. [`Zettel`]s belonging to the same topic are linked forming a tree.
//! This hierarchy of [`Zettel`]s allow to record the development of ideas or knowledge.
//!
//! The trees are collected into something called a [`Kasten`]. A [`Kasten`] keeps track of all
//! [`Zettel`]s and their relations to each other.
//!
//! ## Example
//!
//! ```rust
//! use dingsbums::{Kasten, Zettel};
//!
//! let mut kasten = Kasten::new();
//!
//! // Create a `Zettel`.
//! let root_zettel = Zettel::new("parent".to_string());
//! let root_id = root_zettel.meta_data.id.clone();
//!
//! // Add `Zettel` to `Kasten`, but don't link it other `Zettel`s. That means it's a root
//! // `Zettel`.
//! kasten.add_zettel(root_zettel, vec![]).unwrap();
//!
//! // Create another `Zettel`.
//! let child_zettel = Zettel::new("child".to_string());
//!
//! // Add `Zettel` to `Kasten`. Link it to the root `Zettel` created earlier.
//! kasten.add_zettel(child_zettel, vec![root_id]).unwrap();
//! ```
//!
//! The name "[Dingsbums][dingsbums]" comes from German. It loosely translates to "thingy" in
//! English. The word often used by a person who wants to describe something but forgot the name of
//! the subject.
//!
//! With this project you can document your knowledge in order to recall it at a
//! later point in time. So you don't need to use the word "thingy" or "dingsbums"
//! anymore.
//!
//! [dingsbums]: https://en.wiktionary.org/wiki/Dingsbums
//! [luhman]: https://en.wikipedia.org/wiki/Niklas_Luhmann
//! [zettelkasten-wiki]: https://en.wikipedia.org/wiki/Zettelkasten
pub mod errors;
pub mod kasten;
pub mod zettel;

pub use kasten::Kasten;
pub use zettel::Zettel;
