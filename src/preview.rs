use std::cell::RefCell;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child as ChildProcess, Command, Stdio};

use failure::bail;
use latex;
use tempfile;
use outparse;
use log::{warn, error, info, trace};

use crate::config::AppConfig;
use crate::course::Config;
use crate::latexdoc;
use crate::TeachResult;


pub struct Previewer<'a> {
    root: &'a Path,
    problem: &'a str,
    config: &'a Config,
    temp_dir: RefCell<Option<tempfile::TempDir>>,
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
        trace!("Creating temporary directory");
        if let Some(_) = self.temp_dir.borrow().as_ref() {
            return Ok(());
        }
        self.temp_dir.replace(Some(tempfile::TempDir::new()?));
        Ok(())
    }

    fn create_preview_doc(&self) -> latex::Document {
        trace!("Creating preview document");
        latexdoc::make_preview_sheet(&self.problem, &self.config.sheet_config)
    }

    fn create_latex_command(&self) -> TeachResult<Command> {
        let app_config = AppConfig::get();

        trace!("Creating latex command");
        trace!("TeX engine: {}", &app_config.tex_engine);

        let mut cmd = Command::new(&app_config.tex_engine);
        cmd.stdin(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        if let Some(dir) = self.temp_dir.borrow().as_ref() {
            cmd.current_dir(&dir.path());
        } else {
            bail!("Temporary directory not created")
        }
        let path = self.root.canonicalize()?;
        let problems_path = path.join(&self.config.sources.problems);
        let include_path = path.join("include");
        trace!("Problems: {}", problems_path.display());
        trace!("Include: {}", include_path.display());

        let texinputs = format!("{}:{}:", problems_path.display(), include_path.display());
        trace!("{}", texinputs);

        cmd.env("TEXINPUTS", texinputs);

        Ok(cmd)
    }

    fn create_pdf(&self) -> TeachResult<()> {
        trace!("Creating preview PDF file");
        self.create_temp_dir()?;
        let mut cmd = self.create_latex_command()?;

        let doc = self.create_preview_doc();
        trace!("{:?}", &cmd);

        let mut child: ChildProcess;
        for i in 0..2 {
            trace!("Build {}", i);
            child = cmd.spawn()?;

            if let Some(ref mut stdin) = child.stdin {
                stdin.write(latex::print(&doc)?.as_bytes())?;
            } else {
                bail!("Something went horribly wrong!")
            }

            child.wait()?;
            if let Some(stdout) = child.stdout {
                let rd = BufReader::new(stdout);
                let report = outparse::parse_log(rd);
                info!("{}: {}", &self.problem, &report);
                for message in &report.messages {
                    use outparse::Message::*;
                    match message {
                        Error(i) => warn!("Error: {}", i.full),
                        Warning(i) => info!("Warning: {}", i.full),
                        Badbox(i) => trace!("Badbox: {}", i.full),
                        MissingCitation { label } => info!("Missing citation: {}", label),
                        MissingReference { label } => info!("Missing reference: {}", label),
                        _ => {}
                    };
                }

                if report.missing_references == 0 && report.missing_citations == 0 {
                    break
                }
            }
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
