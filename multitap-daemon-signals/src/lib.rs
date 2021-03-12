use multitap_daemon_core::eventloop::prelude::*;

use std::os::unix::io::AsRawFd;

use nix::sys::{
    signal::{self, SigSet},
    signalfd::{SignalFd, SfdFlags},
};
use mio::unix::SourceFd;

mod error;
pub use error::{Error, Result};

pub struct Signals(SignalFd);

impl Signals {
    pub fn new() -> Result<Signals> {
        let mut mask = SigSet::empty();
        mask.add(signal::SIGHUP);
        mask.add(signal::SIGTERM);
        mask.thread_block()?; 
        Ok(Signals(SignalFd::with_flags(&mask, SfdFlags::SFD_NONBLOCK)?))
    }
}

impl<P: Platform> EventSource<P> for Signals {

    type Error = Error;

    fn register<H: EventHandler>(&mut self, registry: &Registry, token: Token, _: &mut H) -> Result<()> {
        SourceFd(&self.0.as_raw_fd()).register(registry, token, Interest::READABLE)?;
        Ok(())
    }

    fn deregister<H: EventHandler>(&mut self, registry: &Registry, _: &mut H) -> Result<()> {
        SourceFd(&self.0.as_raw_fd()).deregister(registry)?;
        Ok(())
    }

    fn read_event<H: EventHandler>(&mut self, _: &mut H) -> Result<EventProcessing<<H as EventHandler>::Platform>> {
        const SIGHUP:  u32 = signal::SIGHUP  as _;
        const SIGTERM: u32 = signal::SIGTERM as _;

        match self.0.read_signal()? {
            Some(signal) => match signal.ssi_signo {
                SIGHUP  => Ok(EventProcessing::Reload),
                SIGTERM => Ok(EventProcessing::Term),
                _       => panic!("Unexpected signal from signalfd"),
            },
            None => Ok(EventProcessing::Done),
        }
    }
}
