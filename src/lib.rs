


use failure::Error;


pub type TeachResult<T> = Result<T, Error>;

pub mod course;
pub mod latexdoc;
pub mod config;

pub use course::CourseFile;
