use multitap_daemon_core::eventloop::prelude::*;

use std::io::{self, Read};
use std::path::Path;

use mio::net::{UnixListener, UnixStream};

use multitap_core::protocol;

mod error;
pub use error::{Error, Result};
use snafu::ResultExt;

const PATH: &str = multitap_core::protocol::DEFAULT_SOCK;
const SOCKET_CONNECTION_BUF_SIZE: usize = 1024;
pub struct CommandsListener(UnixListener);
pub struct CommandsClient {
    stream: UnixStream,
    buf:    Box<[u8; SOCKET_CONNECTION_BUF_SIZE]>,
    index:  usize,
}

pub struct ClientWriteFactory<'a> {
    stream: &'a UnixStream,
}

pub struct ClientWrite {
    stream: UnixStream,
}

impl CommandsListener {
    pub fn remove_file() {
        use std::io::ErrorKind::NotFound;
        match std::fs::remove_file(PATH) {
            Ok(()) => (),
            Err(err) if err.kind() == NotFound => (),
            Err(err) => Err(err).unwrap(),
        }
    }

    pub fn new() -> Result<CommandsListener> {
        use std::io::ErrorKind::NotFound;
        use std::os::unix::fs::PermissionsExt;

        let path = Path::new(PATH);
        std::fs::create_dir_all(path.parent().unwrap()).with_context(|| error::CreateSocketDir { path })?;
        match std::fs::remove_file(path) {
            Ok(_) => (),
            Err(ref err) if err.kind() == NotFound => (),
            Err(err) => Err(err).with_context(|| error::RemoveExistingSocket { path })?,
        }

        let socket = CommandsListener(UnixListener::bind(PATH).context(error::BindSocket)?);
        std::fs::set_permissions(PATH, std::fs::Permissions::from_mode(0o666)).context(error::SetSocketPermission)?;

        Ok(socket)
    }
}

impl<P: Platform> EventSource<P> for CommandsListener
where
    P::Source: From<CommandsClient>,
{

    type Error = Error;

    fn register<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, token: Token, _: &mut H) -> Result<()> {
        self.0.register(registry, token, Interest::READABLE).context(error::MioSocketRegister)
    }

    fn deregister<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, _: &mut H) -> Result<()> {
        self.0.deregister(registry).context(error::MioSocketDeregister)
    }

    fn read_event<H: EventHandler<Platform = P>>(&mut self, _: &mut H) -> Result<EventProcessing<P>> {
        match self.0.accept() {
            Ok((stream, _addr)) => {
                let client = CommandsClient {
                    stream,
                    buf: Box::new([0; SOCKET_CONNECTION_BUF_SIZE]),
                    index: 0
                };
                let client = P::Source::from(client);
                Ok(EventProcessing::Register(client))
            },
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(EventProcessing::Done),
            Err(e) => Err(e).context(error::AcceptFromSocket)?,
        }
    }
}

impl<P: Platform> EventSource<P> for CommandsClient
where 
    P: Platform,
    P::ClientWrite: From<ClientWrite>,
{
    type Error = Error;

    fn register<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, token: Token, _: &mut H) -> Result<()> {
        self.stream.register(registry, token, Interest::READABLE).context(error::MioClientRegister)
    }

    fn deregister<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, _: &mut H) -> Result<()> {
        self.stream.deregister(registry).context(error::MioClientDeregister)
    }

    fn read_event<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<EventProcessing<P>> {
        use std::io::Write;
        use multitap_core::json::{
            from_slice as json_read,
            to_writer as json_write,
        };
        use std::io::ErrorKind::WouldBlock;

        let result = match self.stream.read(&mut self.buf[self.index..]) {
            Ok(0) => return Ok(EventProcessing::Done),
            Ok(size) => {
                self.index += size;
                match self.buf[self.index-1] {
                    protocol::DELIMITER => {
                        trace!("JSON Read: {}", std::str::from_utf8(&self.buf[0..(self.index-1)]).unwrap());
                        json_read(&self.buf[0..(self.index-1)])
                    },
                    _ => {
                        trace!("[UDS] Receiving message, partial read");
                        return Ok(EventProcessing::Continue)
                    },
                }
            },
            Err(ref err) if err.kind() == WouldBlock => return Ok(EventProcessing::Done),
            Err(err) => return Err(error::Error::ReadFromSocket { source: err }),
        };

        self.index = 0;
        match result {
            Ok(command) => {

                use protocol::Command::*;
                match command {
                    Reload  => Ok(EventProcessing::Reload),
                    Quit    => Ok(EventProcessing::Term),
                    command => { 
                        let client_write_factory = ClientWriteFactory { stream: &mut self.stream };
                        let response = handler.on_command(client_write_factory, command);
                        trace!("JSON Write: {}", multitap_core::json::to_string(&response).context(error::WritingResponse)?);
                        json_write(&mut self.stream, &response).context(error::WritingResponse)?;
                        self.stream.write(&[protocol::DELIMITER]).context(error::WritingEndMarker)?;
                        Ok(EventProcessing::Continue)
                    },
                }
            },
            Err(err) if err.is_eof() => Ok(EventProcessing::Done),
            Err(err) => Err(err).context(error::DecodeCommand)?,
        }
    }
}

impl multitap_daemon_core::ClientWrite for ClientWrite {
    type Error = Error;
    fn write(&mut self, response: &protocol::Event) -> Result<(), Self::Error> {
        use std::io::Write;
        use multitap_core::json::to_writer as json_write;

        json_write(&mut self.stream, &response).context(error::WritingResponse)?;
        self.stream.write(&[protocol::DELIMITER]).context(error::WritingEndMarker)?;
        Ok(())
    }
}

impl<'a, P: Platform> multitap_daemon_core::ClientWriteFactory<P> for ClientWriteFactory<'a>
where
    P::ClientWrite: From<ClientWrite>,
{
    fn create(&self) -> P::ClientWrite {
        use std::os::unix::io::{AsRawFd, FromRawFd};
        let fd = self.stream.as_raw_fd();
        let stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) };
        let stream = UnixStream::from_std(stream);

        P::ClientWrite::from(ClientWrite { stream })
    }
}
