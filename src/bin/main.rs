use clap::Clap;
use uuid::Uuid;
use zettelkasten::kasten::Kasten;
use zettelkasten::zettel::Zettel;

#[derive(Debug, Clap)]
struct Opts {
    #[clap(short, long, default_value = ".zettelkasten")]
    path: String,

    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    #[clap(about = "Initialize new ZellelKasten.")]
    Init,

    #[clap(about = "Create new Zettel.")]
    New(New),
}

#[derive(Clap, Debug)]
struct New {
    #[clap(about = "ID of parent.")]
    parent: Option<Uuid>,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcommand {
        SubCommand::Init => {
            let kasten = Kasten::new();
            kasten
                .to_dir(&opts.path)
                .expect("Failed to initialize new ZettelKasten.");
        }
        SubCommand::New(New { parent }) => {
            let mut kasten =
                Kasten::from_dir(&opts.path).expect("Failed to restore ZettelKasten from disk.");

            let mut parents = vec![];
            let zettel = match parent {
                Some(id) => {
                    parents.push(id);
                    Zettel::new("Child zettel".to_string())
                }
                None => Zettel::new("Root zettel".to_string()),
            };
            kasten
                .add_zettel(zettel, parents)
                .expect("Failed to add Zettel.");
            kasten
                .to_dir(&opts.path)
                .expect("Failed to store ZettelKasten.");
        }
    }
}
