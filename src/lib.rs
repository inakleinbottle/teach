use failure::Error;

pub type TeachResult<T> = Result<T, Error>;

pub mod config;
pub mod course;
pub mod course_items;
pub mod latexdoc;
pub mod makefile;
pub mod preview;

pub use course::Course;
