#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Nix(nix::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error { Error::IO(err) }
}


impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Error { Error::Nix(err) }
}
