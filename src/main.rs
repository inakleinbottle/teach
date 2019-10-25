use std::path::PathBuf;

use log::{self, info};
use simple_logger;
use structopt::StructOpt;
use term_grid;
use term_size;

use teach::preview::Previewer;
use teach::{Course, TeachResult};

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

    #[structopt(name = "course")]
    CourseCmd,

    #[structopt(name = "problems")]
    Problems { problems: Vec<String> },
}

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "p", long = "path", parse(from_os_str), default_value = ".")]
    path: PathBuf,

    #[structopt(short = "v", long = "verbose", conflicts_with = "quiet")]
    verbose: bool,

    #[structopt(short = "q", long = "quiet", conflicts_with = "verbose")]
    quiet: bool,

    #[structopt(flatten)]
    command: Commands,
}

fn main() -> TeachResult<()> {
    let opt = Options::from_args();

    let cf = Course::load(&opt.path)?;
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
            cf.build()?;
        }
        Problem(info) => {
            info!("Editing problem {}", &info.name);
            cf.edit_problem(&info.name, info.touch)?;
        }
        Solution(info) => {
            info!("Editing solution {}", &info.name);
            cf.edit_solution(&info.name, info.touch)?;
        }
        Preview { name } => {
            info!("Previewing problem {}", name);
            let previewer = Previewer::new(&cf.path, &name, &cf.course_file.config);
            previewer.preview()?;
        }
        CourseCmd => {
            info!("Editing course file {}", cf.path.display());
            cf.edit_course_file()?;
        }
        Problems { problems } => {
            let mut grid = term_grid::Grid::new(term_grid::GridOptions {
                direction: term_grid::Direction::LeftToRight,
                filling: term_grid::Filling::Spaces(1),
            });

            cf.get_problems(&problems)?.into_iter().for_each(|prob| {
                grid.add(term_grid::Cell::from(prob));
            });

            if let Some((w, _)) = term_size::dimensions() {
                println!("Printing problems");
                println!("{}", grid.fit_into_width(w).unwrap());
            }
        }
    }

    Ok(())
}
