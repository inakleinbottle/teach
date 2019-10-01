use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::fs;

use latex;
use log::info;
use serde::{Deserialize, Serialize};

use crate::TeachResult;
use crate::latexdoc::{make_coursework_sheet, make_problem_sheet};
use crate::makefile::{write_component_makefile, write_sheet_makefile};
use crate::Course;


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
        course: &Course
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
                        &course.year,
                        &course.course_file.metadata,
                        &info.problems,
                        &course.course_file.config.sheet_config,
                    ))?,
                )?;
                fs::write(
                    root.join(format!("{}-solutions.tex", name)),
                    latex::print(&make_problem_sheet(
                        &format!("{} -- Solutions", &info.title),
                        intro,
                        &course.year,
                        &course.course_file.metadata,
                        &info.problems,
                        &course.course_file.config.solution_config,
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
                        &course.year,
                        &course.course_file.metadata,
                        &info.problems,
                        &info.marks,
                        &course.course_file.config.sheet_config,
                    ))?,
                )?;
                fs::write(
                    root.join(format!("{}-solutions.tex", name)),
                    latex::print(&make_problem_sheet(
                        &format!("{} -- Solutions", &info.title),
                        intro,
                        &course.year,
                        &course.course_file.metadata,
                        &info.problems,
                        &course.course_file.config.solution_config,
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
    pub fn build(
        &self, 
        root: &Path, 
        course: &Course
    ) -> TeachResult<()> {
        for (name, item) in self.items.iter() {
            info!("Creating {}/{}", root.display(), name);
            let path = root.join(name);
            if !path.exists() {
                fs::create_dir(&path)?;
            }
            item.build(
                name, 
                &path, 
                course,
            )?;
        }
        let mut probs_path = PathBuf::from("..");
        probs_path.push("..");
        probs_path.push(&course.course_file.config.sources.problems);
        write_component_makefile(
            root, 
            &probs_path.to_string_lossy(), 
            &["../../include"]
        )?;

        Ok(())
    }
}