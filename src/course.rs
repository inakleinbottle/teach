use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use failure::bail;
use latex;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use toml;

use crate::config::AppConfig;
use crate::latexdoc::{make_coursework_sheet, make_problem_sheet};
use crate::makefile::{write_component_makefile, write_sheet_makefile, write_toplevel_makefile};
use crate::TeachResult;

#[derive(Serialize, Deserialize, Debug)]
pub struct Sources {
    pub problems: String,

    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub sources: Sources,

    #[serde(rename = "sheets", default)]
    pub sheet_config: SheetConfig,

    #[serde(rename = "solutions", default)]
    pub solution_config: SheetConfig,

    #[serde(rename = "courseworks", default)]
    pub coursework_config: SheetConfig,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SheetConfig {
    pub document_class: Option<String>,
    pub problem_macro: Option<String>,
    pub include_preamble: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub author: String,
    pub date: String,

    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl Deref for Metadata {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.other
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SheetInfo {
    pub title: String,
    pub topic: String,
    pub intro: Option<String>,
    pub problems: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CourseworkInfo {
    pub title: String,
    pub topic: String,
    pub intro: Option<String>,
    pub problems: Vec<String>,
    pub marks: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum CourseItem {
    Coursework(CourseworkInfo),
    Sheet(SheetInfo),
}

impl CourseItem {
    fn build(
        &self,
        name: &str,
        root: &Path,
        metadata: &Metadata,
        config: &Config,
    ) -> TeachResult<()> {
        match self {
            Self::Sheet(info) => {
                let intro = match info.intro {
                    Some(ref t) => t,
                    None => "",
                };

                fs::write(
                    root.join(format!("{}-problems.tex", name)),
                    latex::print(&make_problem_sheet(
                        &info.title,
                        intro,
                        metadata,
                        &info.problems,
                        &config.sheet_config,
                    ))?,
                )?;
                fs::write(
                    root.join(format!("{}-solutions.tex", name)),
                    latex::print(&make_problem_sheet(
                        &format!("{} -- Solutions", &info.title),
                        intro,
                        metadata,
                        &info.problems,
                        &config.solution_config,
                    ))?,
                )?;
                write_sheet_makefile(name, root, &info.problems)?;
            }

            Self::Coursework(info) => {
                let intro = match info.intro {
                    Some(ref t) => t,
                    None => "",
                };

                fs::write(
                    root.join(format!("{}-problems.tex", name)),
                    latex::print(&make_coursework_sheet(
                        &info.title,
                        intro,
                        metadata,
                        &info.problems,
                        &info.marks,
                        &config.sheet_config,
                    ))?,
                )?;
                fs::write(
                    root.join(format!("{}-solutions.tex", name)),
                    latex::print(&make_problem_sheet(
                        &format!("{} -- Solutions", &info.title),
                        intro,
                        metadata,
                        &info.problems,
                        &config.solution_config,
                    ))?,
                )?;
                write_sheet_makefile(name, root, &info.problems)?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Component {
    #[serde(flatten)]
    pub items: HashMap<String, CourseItem>,
}

impl Component {
    fn build(&self, root: &Path, metadata: &Metadata, config: &Config) -> TeachResult<()> {
        for (name, item) in self.items.iter() {
            info!("Creating {}/{}", root.display(), name);
            let path = root.join(name);
            if !path.exists() {
                fs::create_dir(&path)?;
            }
            item.build(name, &path, &metadata, &config)?;
        }
        write_component_makefile(root, &config.sources.problems, &["../include"])?;

        Ok(())
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
        let p = path.join("course.toml");
        let cf = toml::from_str(&fs::read_to_string(&p)?)?;
        Ok(cf)
    }

    pub fn build(&self, path: &Path) -> TeachResult<()> {
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
            items.build(&p, &self.metadata, &self.config)?;
        }

        let components: Vec<&String> = self.items.keys().collect();
        write_toplevel_makefile(path, components.as_slice())?;

        Ok(())
    }

    fn edit(&self, path: &Path, problem: &str, component: &str, touch: bool) -> TeachResult<()> {
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

    pub fn edit_problem(&self, path: &Path, problem: &str, touch: bool) -> TeachResult<()> {
        self.edit(path, problem, "problem.tex", touch)
    }

    pub fn edit_solution(&self, path: &Path, problem: &str, touch: bool) -> TeachResult<()> {
        self.edit(path, problem, "solution.tex", touch)
    }
}
