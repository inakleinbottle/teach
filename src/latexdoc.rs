use std::convert::Into;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use latex::{
        DocumentClass, 
        PreambleElement, 
        Element, 
        Document
    };

use crate::course::{Metadata, SheetConfig};


struct Enumerate(Vec<String>);

impl Deref for Enumerate {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Enumerate {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<Element> for Enumerate {

    fn into(self) -> Element {
        let items = self.0.iter().map(|i| {
            format!("\\item {}", i)
        }).collect();
        Element::Environment(
            "enumerate".to_owned(),
            items
        )
    }

}



fn make_basic_doc(
        doc_class: &str, 
        title: &str,
        metadata: &Metadata
    ) -> Document {
    let document_class = match doc_class {
        "article" => DocumentClass::Article,
        "report" => DocumentClass::Report,
        "book" => DocumentClass::Book,
        other => DocumentClass::Other(other.to_owned())
    };

    let mut doc = Document::new(document_class);

    
    doc.preamble.author(&metadata.author)
                .title(title);
    doc.preamble.push(
        PreambleElement::UserDefined(
            format!("\\date{{{}}}", metadata.date)
        )
    );

    doc
}


pub fn make_problem_sheet<S: AsRef<str>>(
    title: &str,
    metadata: &Metadata,
    problems: &[S],
    sheet_config: &SheetConfig
) -> Document {
    let doc_class = match sheet_config.document_class.as_ref() {
        Some(ref cls) => cls,
        None => "article"
    };

    let mut doc = make_basic_doc(&doc_class, title, metadata);

    let problem_macro = match sheet_config.problem_macro.as_ref() {
        Some(ref mac) => mac,
        None => "\\input"
    };

    doc.push(
        Element::UserDefined("\\maketitle".to_owned())
    );

    doc.push(
        Element::Environment(
            "enumerate".to_owned(),
            problems.iter().map(|item| {
                format!("\\{}{{{}}}", problem_macro, item.as_ref())
            }).collect()
        )
    );

    doc
}

pub fn make_solution_sheet<S: AsRef<str>>(
    title: &str,
    metadata: &Metadata,
    problems: &[S],
    sheet_config: &SheetConfig
) -> Document {
    let doc_class = match sheet_config.document_class.as_ref() {
        Some(ref cls) => cls,
        None => "article"
    };

    let mut doc = make_basic_doc(&doc_class, title, metadata);

    let problem_macro = match sheet_config.problem_macro.as_ref() {
        Some(ref mac) => mac,
        None => "\\input"
    };

    doc.push(
        Element::UserDefined("\\maketitle".to_owned())
    );

    doc.push(
        Element::Environment(
            "enumerate".to_owned(),
            problems.iter().map(|item| {
                format!("\\{}{{{}}}", problem_macro, item.as_ref())
            }).collect()
        )
    );

    doc
}
