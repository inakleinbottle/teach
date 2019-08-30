


use failure::Error;


pub type TeachResult<T> = Result<T, Error>;

pub mod course;

pub use course::CourseFile;
