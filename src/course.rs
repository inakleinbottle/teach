use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};


use failure::bail;
use serde::{Serialize, Deserialize};
use toml;
use structopt::StructOpt;

use crate::TeachResult;

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    author: String,
    date: String,
    
    #[serde(flatten)]
    other: HashMap<String, String>,

}


#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum CourseItem {
    Sheet { 
        title: String,
        topic: String, 
        problems: HashMap<String, usize>
    },

}


impl CourseItem {
    
    fn write(&self, name: &str, root: &Path, metadata: &Metadata) -> TeachResult<()> {
        match self {

            Self::Sheet { title, topic, problems } => {

                let path = root.join(format!("{}.tex", name));

                let mut file = fs::File::create(&path)?;

                writeln!(file, "\\documentclass[11pt]{{article}}\n\n")?;
                writeln!(file, "\\def\\topic#1{{}}")?;
                writeln!(file, "\\newcommand*\\includeproblem[1]{{%
    \\item\\input{{#1/problem}}}}")?;
    



                writeln!(file, "\\title{{{}}}", title)?;
                
                writeln!(file, "\\author{{{}}}", metadata.author)?;
                writeln!(file, "\\date{{{}}}", metadata.date)?;
                for (k, v) in metadata.other.iter() {
                    writeln!(file, "\\{}{{{}}}", k, v)?;
                }

                writeln!(file, "\\topic{{{}}}\n\n", topic)?;

                writeln!(file, "\\begin{{document}}")?;
                writeln!(file, "\\begin{{enumerate}}\n")?;
                if problems.is_empty() {
                    writeln!(file, "\\item")?;
                }
                for (problem, mark) in problems.iter() {
                    writeln!(file, "\\includeproblem{{{}}}", problem)?;
                }
                writeln!(file, "\\end{{enumerate}}")?;
                writeln!(file, "\\end{{document}}")?;
            },

        }
        Ok(())
    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Component(HashMap<String, CourseItem>);

impl Component {

    fn write(&self, root: &Path, metadata: &Metadata) -> TeachResult<()> {
        for (n, item) in self.0.iter() {
             println!("Creating {}/{}", root.display(), n);
             item.write(n, root, &metadata)?;
        }
        Ok(())
    }

}


#[derive(Serialize, Deserialize, Debug)]
pub struct CourseFile {
    metadata: Metadata,
    
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
            items.write(&p, &self.metadata)?;

        }

        Ok(())
    }

}
