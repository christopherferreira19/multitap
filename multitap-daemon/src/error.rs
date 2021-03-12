use multitap_core::config;
use crate::plug;

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    Plug(plug::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<config::Error> for Error {
    fn from(error: config::Error) -> Self { Error::Config(error) }
}
impl From<plug::Error> for Error {
    fn from(error: plug::Error) -> Self { Error::Plug(error) }
}
