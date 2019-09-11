use std::cell::RefCell;
use std::io::prelude::*;
use std::process::{Command, Stdio, Child as ChildProcess};
use std::path::{PathBuf, Path};

use failure::bail;
use tempfile;
use latex;

use crate::TeachResult;
use crate::latexdoc;
use crate::course::Config;
use crate::config::AppConfig;

pub struct Previewer<'a> {
    root: &'a Path,
    problem: &'a str,
    config: &'a Config,
    temp_dir: RefCell<Option<tempfile::TempDir>>
}


impl<'a> Previewer<'a> {

    pub fn new(root: &'a Path, problem: &'a str, config: &'a Config) -> Previewer<'a> {
        Previewer {
            root,
            problem,
            config,
            temp_dir: RefCell::new(None),
        }
    }

    fn create_temp_dir(&self) -> TeachResult<()> {
        self.temp_dir.replace(Some(tempfile::TempDir::new()?));
        Ok(())
    }

    fn create_preview_doc(&self) -> latex::Document {
        latexdoc::make_preview_sheet(&self.problem, &self.config.sheet_config)
    }

    fn create_latex_command(&self) -> TeachResult<Command> {
        let app_config = AppConfig::get();
        
        let mut cmd = Command::new(&app_config.tex_engine);
        cmd.stdin(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::inherit());

        if let Some(dir) = self.temp_dir.borrow().as_ref() {
            cmd.current_dir(&dir.path());
        } else {
            bail!("Temporary directory not created")
        }
        let path = self.root.canonicalize()?;
        let problems_path = path.join(&self.config.sources.problems);
        let include_path = path.join("include");
        println!("Problems: {}", problems_path.display());
        println!("Include: {}", include_path.display());

        let texinputs = format!(
            "{}:{}:", problems_path.display(), include_path.display());
        println!("{}", texinputs);

        cmd.env("TEXINPUTS", texinputs);

        Ok(cmd)
    }

    fn create_pdf(&self) -> TeachResult<()> {
        self.create_temp_dir()?;
        let mut cmd = self.create_latex_command()?;

        let doc = self.create_preview_doc();
        println!("{:?}", &cmd);

        let mut child: ChildProcess;
        for i in 0..2 {
            println!("Build {}", i);
            child = cmd.spawn()?;
            

            if let Some(ref mut stdin) = child.stdin {
                stdin.write(latex::print(&doc)?.as_bytes())?;
            } else {
                bail!("Something went horribly wrong!")
            }

            child.wait()?;
        }

        Ok(())
    }

    fn open_viewer(&self) -> TeachResult<()> {
        let app_config = AppConfig::get();
        let mut cmd = Command::new(&app_config.pdf_viewer);
        cmd.stderr(Stdio::null());
        cmd.stderr(Stdio::null());
        if let Some(dir) = self.temp_dir.borrow().as_ref() {
            cmd.current_dir(&dir.path());
        } else {
            bail!("Temporary directory not created")
        }
        cmd.arg("texput.pdf");

        cmd.output()?;

        Ok(())
    }


    pub fn preview(&self) -> TeachResult<()> {
        
        self.create_pdf()?;
        self.open_viewer()
    }
}