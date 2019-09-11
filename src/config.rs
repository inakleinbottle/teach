
use confy;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};


lazy_static!{
    static ref TEACH_CONFIG: AppConfig = confy::load("teach")
                                               .unwrap_or_default();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {

    pub editor: String,
    pub pdf_viewer: String,

    pub tex_engine: String,
    pub tex_flags: String

}

impl AppConfig {

    pub fn get() -> &'static AppConfig {
        &TEACH_CONFIG
    }

}


#[cfg(not(target_os="windows"))]
impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            editor: String::from("vim"),
            pdf_viewer: String::from("evince"),
            tex_engine: String::from("pdflatex"),
            tex_flags: String::from("-interaction=nonstopmode"),
        }
    }
}


#[cfg(target_os="windows")]
impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            editor: String::from("notepad"),
            pdf_viewer: String::from("AcroRd32"),
            tex_engine: String::from("pdflatex"),
            tex_flags: String::from("-interaction=nonstopmode"),
        }
    }
}