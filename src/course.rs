use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::io::prelude::*;
use std::path::{Path, PathBuf};


use failure::bail;
use latex;
use serde::{Serialize, Deserialize};
use toml;


use crate::TeachResult;
use crate::latexdoc::{make_problem_sheet, make_coursework_sheet};
use crate::makefile::{write_sheet_makefile, write_component_makefile};

#[derive(Serialize, Deserialize, Debug)]
pub struct Sources {

    problems: String,

    #[serde(flatten)]
    other: HashMap<String, String>,

}


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {

    pub sources: Sources,

    #[serde(rename="sheet", default)]
    pub sheet_config: SheetConfig,

    #[serde(rename="solution", default)]
    pub solution_config: SheetConfig,

    #[serde(rename="coursework", default)]
    pub coursework_config: SheetConfig,

}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SheetConfig { 
    pub document_class: Option<String>,
    pub problem_macro: Option<String>,
    pub include_preamble: Option<String>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub author: String,
    pub date: String,
    
    #[serde(flatten)]
    other: HashMap<String, String>,
}

impl Deref for Metadata {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.other
    }

}


#[derive(Serialize, Deserialize, Debug)]
pub struct SheetInfo {
    title: String,
    topic: String,
    problems: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CourseworkInfo {
    title: String,
    topic: String,
    problems: Vec<String>,
    marks: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum CourseItem {
    Coursework(CourseworkInfo),
    Sheet(SheetInfo),
}


impl CourseItem {
    
    fn write(
        &self, 
        name: &str, 
        root: &Path, 
        metadata: &Metadata,
        config: &Config,
    ) -> TeachResult<()> {
        match self {

            Self::Sheet(info) => {

                fs::write(
                    root.join(format!("{}-problems.tex", name)),
                    latex::print(
                        &make_problem_sheet(
                            &info.title,
                            metadata,
                            &info.problems,
                            &config.sheet_config
                        )
                    )?
                )?;
                fs::write(
                    root.join(format!("{}-solutions.tex", name)),
                    latex::print(
                        &make_problem_sheet(
                            &info.title,
                            metadata,
                            &info.problems,
                            &config.solution_config
                        )
                    )?
                )?;
                write_sheet_makefile(name, root, &info.problems)?;
               
            },

            Self::Coursework(info) => {

                fs::write(
                    root.join(format!("{}-problems.tex", name)),
                    latex::print(
                        &make_coursework_sheet(
                            &info.title,
                            metadata,
                            &info.problems,
                            &info.marks,
                            &config.sheet_config
                        )
                    )?
                )?;
                fs::write(
                    root.join(format!("{}-solutions.tex", name)),
                    latex::print(
                        &make_problem_sheet(
                            &format!("{} -- Solutions", &info.title),
                            metadata,
                            &info.problems,
                            &config.solution_config
                        )
                    )?
                )?;
            },

        }
        Ok(())
    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Component {

    #[serde(flatten)]
    items: HashMap<String, CourseItem> 
}

impl Component {

    fn write(
        &self, 
        root: &Path, 
        metadata: &Metadata, 
        config: &Config
    ) -> TeachResult<()> {
        
        for (name, item) in self.items.iter() {
             println!("Creating {}/{}", root.display(), name);
             let path = root.join(name);
             if !path.exists() {
                 fs::create_dir(&path)?;
             }
             item.write(name, &path, &metadata, &config)?;
        }
        write_component_makefile(
            root,
            &config.sources.problems,
            &["../include"]
        )?;

        Ok(())
    }

}


#[derive(Serialize, Deserialize, Debug)]
pub struct CourseFile {
    metadata: Metadata,

    #[serde(flatten)]
    config: Config,
    
    #[serde(flatten)]
    items: HashMap<String, Component>

}


impl CourseFile {

    pub fn load(path: &Path) -> TeachResult<CourseFile> {
        let p = path.join("course.toml");
        let cf = toml::from_str(&fs::read_to_string(&p)?)?;
        Ok(cf)
    }

    pub fn write(&self, path: &Path) -> TeachResult<()> {
        if !path.is_dir() {
            bail!("Path {} does not exist or is not a directory", 
                  &path.display());
        }
    
        let mut p: PathBuf;

        for (component, items) in self.items.iter() {
            p = path.join(component);
            println!("Creating {}", p.display());
            if !p.exists() {
                fs::create_dir(&p)?;
            }
            items.write(&p, &self.metadata, &self.config)?;

        }

        Ok(())
    }

}
