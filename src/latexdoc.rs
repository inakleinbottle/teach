

use latex::{
        DocumentClass, 
        PreambleElement, 
        Element, 
        Document,
        Paragraph,
    };

use crate::course::{Metadata, SheetConfig};



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

    for (mac, item) in metadata.iter() {
        doc.preamble.push(
            PreambleElement::UserDefined(
                format!("\\{}{{{}}}", mac, item)
            )
        );
    }

    doc
}


pub fn make_sheet(
    title: &str,
    intro: &str,
    metadata: &Metadata,
    sheet_config: &SheetConfig
) -> Document {
    let doc_class = match sheet_config.document_class.as_ref() {
        Some(ref cls) => cls,
        None => "article"
    };

    let mut doc = make_basic_doc(&doc_class, title, metadata);



    if let Some(ref include_preamble) = sheet_config.include_preamble {
        doc.preamble.push(
            PreambleElement::UserDefined(
                include_preamble.to_owned()
            )
        );
    }

    doc.push(
        Element::UserDefined("\\maketitle".to_owned())
    );

    doc.push(
        Element::Para(Paragraph::from(intro))
    );


    doc
}

pub fn make_problem_sheet<S: AsRef<str>>(
    title: &str,
    intro: &str,
    metadata: &Metadata,
    problems: &[S],
    sheet_config: &SheetConfig
) -> Document {
    let mut doc = make_sheet(title, intro, metadata, sheet_config);

    let problem_macro = match sheet_config.problem_macro.as_ref() {
        Some(ref mac) => mac,
        None => "\\input"
    };

    doc.push(
        Element::Environment(
            "enumerate".to_owned(),
            problems.iter().map(|item| {
                format!("{}{{{}}}", problem_macro, item.as_ref())
            }).collect()
        )
    );
    doc
}

pub fn make_coursework_sheet<S: AsRef<str>>(
    title: &str,
    intro: &str,
    metadata: &Metadata,
    problems: &[S],
    marks: &[u32],
    sheet_config: &SheetConfig
) -> Document {
    let mut doc = make_sheet(title, intro, metadata, sheet_config);
    
    let problem_macro = match sheet_config.problem_macro.as_ref() {
        Some(ref mac) => mac,
        None => "\\input"
    };

    doc.push(
        Element::Environment(
            "enumerate".to_owned(),
            problems.iter().zip(marks.iter()).map(|(item, mark)| {
                format!("{}[{}]{{{}}}", problem_macro, mark, item.as_ref())
            }).collect()
        )
    );
    doc
}