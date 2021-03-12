use std::{fmt, error as std_error};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait GenericError
where
    Self: std_error::Error + Send + Sync + 'static,
{}

impl<T> GenericError for T
where
    T: std_error::Error + Send + Sync + 'static,
{}

#[derive(Debug)]
pub struct Error(Box<dyn GenericError>);

impl<E: GenericError> From<E> for Error {
    fn from(err: E) -> Error {
        let b: Box<dyn GenericError> = Box::new(err);
        Error(b)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.0, f)
    }
}