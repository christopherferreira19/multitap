use super::evdev;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    OsString(std::ffi::OsString),
    Nix(nix::Error),
    UdevNoDevNode,
    EvdevPhys(evdev::phys::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error { Error::IO(err) }
}

impl From<std::ffi::OsString> for Error {
    fn from(err: std::ffi::OsString) -> Error { Error::OsString(err) }
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Error { Error::Nix(err) }
}

impl From<evdev::phys::Error> for Error {
    fn from(err: evdev::phys::Error) -> Error { Error::EvdevPhys(err) }
}

