use clap::Clap;
use skim::prelude::*;
use std::env;
use std::fs;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use uuid::Uuid;
use zettelkasten::errors::AppError;
use zettelkasten::kasten::Kasten;
use zettelkasten::zettel::Zettel;

#[derive(Debug, Clap)]
struct Opts {
    #[clap(short, long, default_value = "~/.zettelkasten")]
    path: String,

    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    #[clap(about = "Initialize new Zettelkasten.")]
    Init,

    #[clap(about = "Show Kasten as graph")]
    Graph,

    #[clap(about = "Modify existing Zettel")]
    Edit,

    #[clap(about = "Create new Zettel.")]
    New(New),
}

#[derive(Clap, Debug)]
struct New {
    #[clap(long)]
    no_parent: bool,
}

/// `App` is a wrapper around `Kasten` that allows a `Kasten` to be imported from and exported to a
/// file system.
struct App {
    kasten: Kasten,

    // Path to a folder on the file system where the `App` is exported to.
    root: PathBuf,
}

impl App {
    /// Create new `App` with an empty `Kasten`.
    ///
    /// # Examples
    ///
    /// ```
    /// let app = App::new("~/.zettelkasten");
    /// ```
    fn new(path: &str) -> Self {
        App {
            kasten: Kasten::new(),
            root: Path::new(&path).into(),
        }
    }

    /// Create an `App` by importing directory at given `path`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let app = App::import("~/.zettelkasten").unwrap();
    /// ```
    pub fn import(path: &str) -> Result<Self, AppError> {
        let path = Path::new(&path);
        let file = File::open(&path.join("db")).map_err(AppError::WriteError)?;
        let kasten = Kasten::import(file)?;
        Ok(App {
            kasten,
            root: path.into(),
        })
    }

    /// Export an `App` to a file system at the given `path`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let app = App::new("~/.zettelkasten");
    /// app.export().unwrap();
    /// ```
    pub fn export(&self) -> Result<(), AppError> {
        let dir = Path::new(&self.root);
        if !dir.exists() {
            create_dir_all(&self.root.clone()).map_err(AppError::WriteError)?;
        }

        let path = Path::new(&self.root).join("db");
        let file = File::create(&path).map_err(AppError::WriteError)?;
        self.kasten.export(file)?;

        for zettel in self.kasten.zettels.values() {
            if !zettel.dirty {
                continue;
            };

            let path = Path::new(&dir).join(zettel.meta_data.id.to_string());
            let file = File::create(&path).map_err(AppError::WriteError)?;
            zettel.export(&file)?
        }

        Ok(())
    }

    /// Add new `Zettel` to `App`. A `Zettel` can be linked to other `Zettel`s by passing their
    /// IDs as `parents`.
    ///
    /// **note**: `App` must be `export`ed to save any changes.
    ///
    /// ```
    /// let app = App::new("~/.zettelkasten");
    /// let parent = app.new_zettel(vec![]).unwrap()
    /// let child = app.new_zettel(vec![parent]).unwrap()
    /// ```
    pub fn new_zettel(&mut self, parents: Vec<Uuid>) -> Result<Uuid, AppError> {
        let zettel = Zettel::new("".to_string());
        let id = zettel.meta_data.id;
        self.kasten.add_zettel(zettel, parents).map(|_| id)
    }

    /// Open `Zettel` with preferred editor to allow modifications.
    ///
    /// **note**: `App` must be `export`ed to save any changes.
    pub fn open_zettel(&mut self, id: Uuid) -> Result<Zettel, AppError> {
        let zettel_path = Path::new(&self.root).join(id.to_string());
        if !zettel_path.exists() {
            return Err(AppError::ZettelDoesntExistsError);
        }

        let mut zettel = Zettel::import(File::open(zettel_path).unwrap()).unwrap();

        let mut temp_path = env::temp_dir();
        temp_path.push(id.to_string());

        fs::write(&temp_path, zettel.body.clone()).expect("Could not create temporary Zettel.");

        // Open temp file with editor
        let editor = env::var("EDITOR").expect("Env var 'EDITOR' not set");
        Command::new(editor)
            .arg(&temp_path)
            .status()
            .expect("Failed to open temporary zettel with EDITOR.");

        // Read content of temp file
        let content = fs::read(&temp_path).expect("Failed to read content temporary Zettel.");
        zettel.update_body(
            str::from_utf8(&content)
                .expect("Failed to read content temporary Zettel")
                .to_string(),
        );
        self.kasten.update_zettel(zettel.clone());

        return Ok(zettel);
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcommand {
        SubCommand::Init => {
            let app = App::new(&opts.path);
            app.export()
                .expect("Failed to initialize new ZettelKasten.");
        }

        SubCommand::Graph => {
            let app = App::import(&opts.path).expect("Failed to restore ZettelKasten from disk.");
            app.kasten.dot();
        }

        SubCommand::New(New { no_parent: true }) => {
            let mut app =
                App::import(&opts.path).expect("Failed to restore ZettelKasten from disk.");
            let id = app
                .new_zettel(vec![])
                .expect("Failed to create new Zettel.");
            app.export().expect("Failed to store ZettelKasten.");
            app.open_zettel(id)
                .expect("Failed to open Zettel in editor.");
            app.export().expect("Failed to store ZettelKasten.");
        }

        SubCommand::New(New { no_parent: false }) => {
            let mut app =
                App::import(&opts.path).expect("Failed to restore ZettelKasten from disk.");

            let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
            app.kasten.meta_data.iter().for_each(|(_, z)| {
                tx_item.send(Arc::new(format!("{} - \u{2063}{}", z.title, z.id)));
            });

            let options = SkimOptionsBuilder::default().build().unwrap();
            let selected_items = Skim::run_with(&options, Some(rx_item))
                .map(|out| out.selected_items)
                .unwrap_or_else(|| Vec::new());

            let parents = selected_items
                .iter()
                .map(|z| {
                    let x = z.output().clone();
                    let v: Vec<&str> = x.split("\u{2063}").collect();
                    dbg!(v.clone());
                    Uuid::parse_str(v.get(1).expect("Failed to get UUID")).unwrap()
                })
                .collect();

            let id = app
                .new_zettel(parents)
                .expect("Failed to create new Zettel.");
            app.export().expect("Failed to store ZettelKasten.");
            app.open_zettel(id)
                .expect("Failed to open Zettel in editor.");
            app.export().expect("Failed to store ZettelKasten.");
        }

        SubCommand::Edit => {
            let mut app =
                App::import(&opts.path).expect("Failed to restore ZettelKasten from disk.");

            let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
            app.kasten.meta_data.iter().for_each(|(_, z)| {
                tx_item.send(Arc::new(format!("{} - \u{2063}{}", z.title, z.id)));
            });

            let options = SkimOptionsBuilder::default().build().unwrap();
            let selected_items = Skim::run_with(&options, Some(rx_item))
                .map(|out| out.selected_items)
                .unwrap_or_else(|| Vec::new());

            let parents: Vec<Uuid> = selected_items
                .iter()
                .map(|z| {
                    let x = z.output().clone();
                    let v: Vec<&str> = x.split("\u{2063}").collect();
                    dbg!(v.clone());
                    Uuid::parse_str(v.get(1).expect("Failed to get UUID")).unwrap()
                })
                .collect();

            app.open_zettel(parents[0])
                .expect("Failed to open Zettel in editor.");
            app.export().expect("Failed to store ZettelKasten.");
        }
    }
}
