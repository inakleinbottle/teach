use std::path::{Path, PathBuf};

use structopt::StructOpt;
use log::{self, info};
use simple_logger;

use teach::preview::Previewer;
use teach::{CourseFile, TeachResult};

#[derive(StructOpt)]
struct EditInfo {
    name: String,

    #[structopt(
        short = "t",
        long = "touch",
        help = "Create but don't open for editing."
    )]
    touch: bool,
}

#[derive(StructOpt)]
enum Commands {
    #[structopt(name = "build")]
    Build,

    #[structopt(name = "problem")]
    Problem(EditInfo),

    #[structopt(name = "solution")]
    Solution(EditInfo),

    #[structopt(name = "preview")]
    Preview { name: String },
}

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "p", long = "path", parse(from_os_str), default_value = ".")]
    path: PathBuf,

    #[structopt(short="v", long="verbose", conflicts_with="quiet")]
    verbose: bool,

    #[structopt(short="q", long="quiet", conflicts_with="verbose")]
    quiet: bool,

    #[structopt(flatten)]
    command: Commands,
}

fn main() -> TeachResult<()> {
    let opt = Options::from_args();

    let cf = CourseFile::load(&opt.path)?;
    use Commands::*;

    let level = match (opt.verbose, opt.quiet) {
        (true, false) => log::Level::Trace,
        (false, true) => log::Level::Error,
        _ => log::Level::Info,
    };
    simple_logger::init_with_level(level).unwrap();

    match opt.command {
        Build => {
            info!("Building course from {}", &opt.path.display());
            cf.build(&opt.path)?;
        }
        Problem(info) => {
            info!("Editing problem {}", &info.name);
            cf.edit_problem(&opt.path, &info.name, info.touch)?;
        }
        Solution(info) => {
            info!("Editing solution {}", &info.name);
            cf.edit_solution(&opt.path, &info.name, info.touch)?;
        }
        Preview { name } => {
            info!("Previewing problem {}", name);
            let previewer = Previewer::new(&opt.path, &name, &cf.config);
            previewer.preview()?;
        }
    }

    Ok(())
}
