use failure::Error;

pub type TeachResult<T> = Result<T, Error>;

pub mod config;
pub mod course;
pub mod latexdoc;
pub mod makefile;
pub mod preview;
pub mod course_items;

pub use course::Course;
