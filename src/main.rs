use std::path::{PathBuf, Path};

use structopt::StructOpt;

use teach::{CourseFile, TeachResult};
use teach::preview::Previewer;


#[derive(StructOpt)]
struct EditInfo{
    name: String,

    #[structopt(short="t", long="touch", 
      help="Create but don't open for editing.")]
    touch: bool,
}

#[derive(StructOpt)]
enum Commands {

    #[structopt(name="build")]
    Build,

    #[structopt(name="problem")]
    Problem(EditInfo),

    #[structopt(name="solution")]
    Solution(EditInfo),

    #[structopt(name="preview")]
    Preview { name: String },

}

#[derive(StructOpt)]
struct Options {
    #[structopt(short="p", long="path", parse(from_os_str), default_value=".")]
    path: PathBuf,

    #[structopt(flatten)]
    command: Commands

}


fn main() -> TeachResult<()> {
    let opt = Options::from_args();
    
    let cf = CourseFile::load(&opt.path)?;
    use Commands::*;

    match opt.command {
        Build => {
            println!("Building course from {}", &opt.path.display());
            cf.build(&opt.path)?;
        },
        Problem(info) => {
            println!("Editing problem {}", &info.name);
            cf.edit_problem(&opt.path, &info.name, info.touch)?;

            
        },
        Solution(info) => {
            println!("Editing solution {}", &info.name);
            cf.edit_solution(&opt.path, &info.name, info.touch)?;
        },
        Preview { name } => {
            println!("Previewing problem {}", name);
            let previewer = Previewer::new(&opt.path, &name, &cf.config);
            previewer.preview()?;
        }

    }
    
    
    Ok(())

}


