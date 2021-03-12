use snafu::Snafu;
use multitap_core::json;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(in super)))]
pub enum Error {
    #[snafu(display("While creating socket directory {}: {}", path.display(), source))]
    CreateSocketDir { path: std::path::PathBuf, source: std::io::Error },
    #[snafu(display("While removing existing socket {}: {}", path.display(), source))]
    RemoveExistingSocket { path: std::path::PathBuf, source: std::io::Error },
    #[snafu(display("While binding socket: {}", source))]
    BindSocket { source: std::io::Error },
    #[snafu(display("While setting socket permission: {}", source))]
    SetSocketPermission { source: std::io::Error },
    #[snafu(display("While registering socket with mio: {}", source))]
    MioSocketRegister { source: std::io::Error },
    #[snafu(display("While deregistering socket with mio: {}", source))]
    MioSocketDeregister { source: std::io::Error },
    #[snafu(display("While registering socket client with mio: {}", source))]
    MioClientRegister { source: std::io::Error },
    #[snafu(display("While deregistering socket client with mio: {}", source))]
    MioClientDeregister { source: std::io::Error },
    #[snafu(display("While accepting from socket: {}", source))]
    AcceptFromSocket { source: std::io::Error },
    #[snafu(display("While reading from socket: {}", source))]
    ReadFromSocket { source: std::io::Error },
    #[snafu(display("While decoding command: {}", source))]
    DecodeCommand { source: json::Error },
    #[snafu(display("While encoding response: {}", source))]
    WritingResponse { source: json::Error },
    #[snafu(display("While encoding response: {}", source))]
    WritingEndMarker { source: std::io::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/*impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error { Error::IO(err) }
}*/

// impl From<rmp_serde::decode::Error> for Error {
//     fn from(err: rmp_serde::decode::Error) -> Error { Error::CmdDecode(err) }
// }
