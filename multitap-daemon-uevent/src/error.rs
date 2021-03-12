use std::io;

use super::evdev;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Nix(nix::Error),
    EvdevPhys(evdev::phys::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::IO(err) }
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Error { Error::Nix(err) }
}

impl From<evdev::phys::Error> for Error {
    fn from(err: evdev::phys::Error) -> Error { Error::EvdevPhys(err) }
}