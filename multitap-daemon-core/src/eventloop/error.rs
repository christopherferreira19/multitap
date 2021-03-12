use std::fmt::Debug;
use super::Platform;
use super::sources;

#[derive(Debug)]
pub enum Error<P: Platform + Debug> {
    Sources(sources::Error),
    Platform(P::Error),
}

impl<P: Platform + Debug> From<sources::Error> for Error<P> {
    fn from(err: sources::Error) -> Self { Error::Sources(err) }
}
