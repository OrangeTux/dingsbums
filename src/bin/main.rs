use clap::Clap;
use skim::prelude::*;
use std::env::var;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;
use zettelkasten::errors::AppError;
use zettelkasten::kasten::Kasten;
use zettelkasten::zettel::Zettel;

#[derive(Debug, Clap)]
struct Opts {
    #[clap(
        short,
        long,
        default_value = "/home/developer/projects/zettelkasten/.zettelkasten"
    )]
    path: String,

    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    #[clap(about = "Initialize new ZellelKasten.")]
    Init,

    #[clap(about = "Show Kasten as graph")]
    Graph,

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
    /// Create new `App` with an empty 'Kasten'.
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
g       let file = File::open(&path.join("db")).map_err(AppError::WriteError)?;
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

    pub fn get_zettel_path(&self, id: Uuid) -> Result<PathBuf, AppError> {
        self.kasten.get_node_index(id)?;

        let mut path = self.root.clone();
        path.push(format!("{}", id));
        Ok(path)
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
            let zettel = Zettel::new("Root zettel".to_string());
            let id = zettel.meta_data.id.clone();
            app.kasten
                .add_zettel(zettel, vec![])
                .expect("Failed to add Zettel.");
            let path = app.get_zettel_path(id).unwrap();
            app.export().expect("Failed to store ZettelKasten.");
            open_zettel(path);
        }
        SubCommand::New(New { no_parent: false }) => {
            let mut app =
                App::import(&opts.path).expect("Failed to restore ZettelKasten from disk.");

            let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
            app.kasten.meta_data.iter().for_each(|(_, z)| {
                tx_item.send(Arc::new(format!("{} >{}", z.title, z.id)));
            });

            let options = SkimOptionsBuilder::default().build().unwrap();
            let selected_items = Skim::run_with(&options, Some(rx_item))
                .map(|out| out.selected_items)
                .unwrap_or_else(|| Vec::new());

            let parents = selected_items
                .iter()
                .map(|z| {
                    let x = z.output().clone();
                    let v: Vec<&str> = x.split(">").collect();
                    dbg!(v.clone());
                    Uuid::parse_str(v.get(1).expect("Failed to get UUID")).unwrap()
                })
                .collect();

            let zettel = Zettel::new("Child zettel".to_string());
            let id = zettel.meta_data.id.clone();
            app.kasten
                .add_zettel(zettel, parents)
                .expect("Failed to add Zettel.");
            let path = app.get_zettel_path(id).unwrap();
            app.export().expect("Failed to store ZettelKasten.");
            open_zettel(path);

            //let mut parents = vec![];
            //let zettel = match parent {
            //Some(id) => {
            //parents.push(id);
            //Zettel::new("Child zettel".to_string())
            //}
            //None => Zettel::new("Root zettel".to_string()),
            //};
        }
    }
}

pub fn open_zettel(path: PathBuf) {
    let editor = var("EDITOR").unwrap();
    dbg!(&path);
    File::create(&path).expect("Could not create file");

    Command::new(editor)
        .arg(&path)
        .status()
        .expect("Something went wrong");

    //dbg!(path.clone());
    //let mut cmd = Command::new("/usr/bin/nvim");
    //cmd.arg(path.to_str().unwrap());
    //cmd.arg("/tmp/src/main.rs");
    //let output = cmd.status();
    //dbg!(output);
}
