use std::marker::PhantomData;

pub mod error;
pub use error::Error;
pub mod sources;
pub use sources::EventSources;

pub mod prelude {
    pub use multitap_core::log::*;

    pub use mio::{
        self,
        Token,
        Interest,
        Registry,
        event::Source as MioSource,
    };
    
    pub use crate::{Platform, DeviceId};
    pub use super::*;
}

use prelude::*;
pub use mio::Token;
use multitap_core::{protocol, input};
use super::DeviceId;
use super::ClientWriteFactory;

pub trait EventSource<P: Platform> {

    type Error;

    fn register<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, token: Token, handler: &mut H) -> Result<(), Self::Error>;

    fn deregister<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, handler: &mut H) -> Result<(), Self::Error>;

    fn read_event<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<EventProcessing<P>, Self::Error>;
}

pub trait EventHandler: Sized {
    type Platform: Platform;

    fn on_command<CF: ClientWriteFactory<Self::Platform>>(&mut self, client: CF, command: protocol::Command) -> protocol::Result;

    fn on_device_add(&mut self, device: <Self::Platform as Platform>::PhysDevice) -> Option<DeviceId>;

    fn on_device_remove(&mut self, device_index: DeviceId) -> ();

    fn on_input_sync(&mut self, device_index: DeviceId, event: input::SyncEvent);

    fn on_input_key_change(&mut self, device_index: DeviceId, event: input::KeyEvent);

    fn on_input_axis_change(&mut self, device_index: DeviceId, event: input::AxisEvent);

    fn on_input_motion(&mut self, device_index: DeviceId, event: input::MotionEvent);
}

pub enum EventProcessing<P: Platform> {
    Continue,
    Register(P::Source),
    Done,
    Reload,
    Term,
}

pub enum EventLoopResult {
    Reload,
    Term,
}

pub struct EventLoop<P>
where 
    P: Platform,
    Error<P>: From<P::Error>,
{
    _phantom: PhantomData<P>,
}

impl<P> EventLoop<P>
where 
    P: Platform,
    Error<P>: From<P::Error>,
{
    pub fn new() -> Result<Self, Error<P>> {
        let _phantom = PhantomData;
        Ok(Self { _phantom })
    }

    pub fn run<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<EventLoopResult, Error<P>> {
        let mut sources = EventSources::new()?;
        P::initialize(&mut sources, handler)?;

        let mut events = mio::Events::with_capacity(1024);
        loop {
            let mut reload = false;
            let mut term = false;
            sources.poll(&mut events)?;

            for event in &events {
                if event.is_readable() {
                    loop {
                        let result = sources[event.token()].read_event(handler)?;
                        use EventProcessing::*;
                        match result {
                            Continue    => (),
                            Register(r) => { sources.register(r, handler)?; },
                            Done        => break,
                            Reload      => reload = true,
                            Term        => term = true,
                        }
                    }
                }

                if event.is_read_closed() {
                    sources.deregister(event.token(), handler)?;
                }
            }

            if term {
                return Ok(EventLoopResult::Term)
            }
            else if reload {
                return Ok(EventLoopResult::Reload)
            }
        }
    }
}

