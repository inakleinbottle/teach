

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
struct AppConfig {

    editor: String,
    pdf_viewer: String,

}


#[cfg(not(target_os="windows"))]
impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            editor: String::from("vim"),
            pdf_viewer: String::from("evince"),
        }
    }
}


#[cfg(target_os="windows")]
impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            editor: String::from("notepad"),
            pdf_viewer: String::from("AcroRd32")
        }
    }
}