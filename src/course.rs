use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use failure::bail;

use chrono::{self, Datelike};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use toml;

use crate::config::AppConfig;
use crate::course_items::{Metadata, Component, Config};
use crate::makefile::{write_toplevel_makefile};
use crate::TeachResult;


#[derive(Debug)]
pub struct Course {
    pub year: String,
    pub path: PathBuf,
    pub course_file: CourseFile,
}

impl Course {

    fn get_current_academic_year() -> String {
        let date= chrono::Local::today();
        let curr_year = date.year();
        if date.month() > 7 {
            format!("{}-{}", curr_year, (curr_year + 1) % 100)
        } else {
            format!("{}-{}", curr_year - 1, curr_year % 100)
        }
    }

    pub fn load(path: &Path) -> TeachResult<Course> {
        let mut p: PathBuf;
        let year = Course::get_current_academic_year();
        for par in path.canonicalize()?.ancestors() {
            p = par.join("course.toml");
            if p.is_file() {
                return Ok(Course {
                    course_file: CourseFile::load(&p)?,
                    year,
                    path: par.to_owned()
                })
            }
        }
        bail!("Unable to load Course file")
    }

    pub fn edit_problem(&self, problem: &str, touch: bool) -> TeachResult<()> {
        self.course_file.edit(&self.path, problem, "problem.tex", touch)
    }

    pub fn edit_solution(&self, problem: &str, touch: bool) -> TeachResult<()> {
        self.course_file.edit(&self.path, problem, "solution.tex", touch)
    }

    pub fn build(&self) -> TeachResult<()> {
        self.course_file.build(&self.path.join(&self.year), &self.year)
    }

}



#[derive(Serialize, Deserialize, Debug)]
pub struct CourseFile {
    pub metadata: Metadata,

    #[serde(flatten)]
    pub config: Config,

    #[serde(flatten)]
    pub items: HashMap<String, Component>,
}

impl CourseFile {


    pub fn load(path: &Path) -> TeachResult<CourseFile> {
        let cf = toml::from_str(&fs::read_to_string(path)?)?;
        Ok(cf)
    }

    fn build(&self, path: &Path, date: &str) -> TeachResult<()> {
        if !path.is_dir() {
            bail!(
                "Path {} does not exist or is not a directory",
                &path.display()
            );
        }

        let mut p: PathBuf;

        for (component, items) in self.items.iter() {
            p = path.join(component);
            info!("Creating {}", p.display());
            if !p.exists() {
                fs::create_dir(&p)?;
            }
            items.build(&p, date, &self.metadata, &self.config)?;
        }

        let components: Vec<&String> = self.items.keys().collect();
        write_toplevel_makefile(path, components.as_slice())?;

        Ok(())
    }

    fn edit(
        &self, 
        path: &Path, 
        problem: &str, 
        component: &str, 
        touch: bool
    ) -> TeachResult<()> {
        let p = path.join(&self.config.sources.problems);

        if !p.is_dir() && !p.is_file() {
            warn!("Directory {} does not exist, creating", p.display());
            fs::create_dir(&p)?;
        } else if p.is_file() {
            bail!("Path {} is a file", p.display());
        }

        let prob_path = p.join(problem);

        if !prob_path.exists() {
            // Create new problem
            fs::create_dir(&prob_path)?;
            fs::File::create(&prob_path.join("problem.tex"))?;
            fs::File::create(&prob_path.join("solution.tex"))?;
        } else if prob_path.is_file() {
            bail!("Cannot create {}, exists as file", prob_path.display());
        }

        if touch {
            return Ok(());
        }

        Command::new(&AppConfig::get().editor)
            .arg(&prob_path.join(component))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .output()?;

        Ok(())
    }



}
