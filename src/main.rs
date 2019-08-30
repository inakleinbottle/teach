use std::path::{PathBuf, Path};

use structopt::StructOpt;

use teach::{CourseFile, TeachResult};

#[derive(StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    path: PathBuf

}


fn main() -> TeachResult<()> {
    let opt = Options::from_args();
    let cf = CourseFile::load(&opt.path)?;
    
    cf.write(&opt.path)?;
    Ok(())

}


